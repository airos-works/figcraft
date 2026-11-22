//! Present egui's actual per-viewport widget tree to the air/OS accessibility
//! server. Native requests use the same AccessKit actions as other desktops.
use crate::host;
use accesskit_consumer::{NodeRef, Tree, common_filter};
use egui::accesskit::{Action, ActionRequest, NodeId, Toggled, TreeId, TreeUpdate};

const ACTIONS: &[(u32, Action)] = &[
    (1, Action::Click),
    (2, Action::Focus),
    (4, Action::Increment),
    (8, Action::Decrement),
    (32, Action::Expand),
    (64, Action::Collapse),
    (128, Action::ShowContextMenu),
    (256, Action::ScrollIntoView),
];
pub fn request(bit: i32, node: u64) -> Option<ActionRequest> {
    let action = if bit == 16 { Action::Click } else { ACTIONS.iter().find(|(b, _)| *b == bit as u32)?.1 };
    Some(ActionRequest { action, target_tree: TreeId::ROOT, target_node: NodeId(node), data: None })
}
pub struct Item {
    pub id: u64,
    pub parent: u64,
    pub actions: u32,
    pub states: u64,
    pub role: String,
    pub name: String,
    pub value: String,
    pub description: String,
    pub rect: [f32; 4],
}
pub fn publish(window: i32, update: TreeUpdate, scale: f32) {
    // egui supplies a full tree each pass. The consumer resolves inherited
    // transforms, labelled-by relations, and text runs before crossing the FFI.
    let tree = Tree::new(update, true);
    let root = tree.state().root();
    let mut stack = vec![(root, root.locate().0.0)];
    let mut items = Vec::new();
    while let Some((node, parent)) = stack.pop() {
        let id = node.locate().0.0;
        let mut actions = ACTIONS.iter().filter(|(_, a)| node.data().supports_action(*a)).fold(0, |v, (b, _)| v | *b);
        if node.is_selectable() && node.data().supports_action(Action::Click) {
            actions |= 16;
        }
        let bounds = node.bounding_box().unwrap_or_default();
        let rect = [bounds.x0, bounds.y0, bounds.x1, bounds.y1].map(|v| v as f32 / scale);
        let protected = node.is_password_input();
        let name = node.label().or_else(|| node.data().placeholder().map(str::to_owned)).unwrap_or_default();
        items.push(Item {
            id,
            parent,
            actions,
            states: states(&node),
            role: format!("{:?}", node.role()),
            name,
            description: node.description().unwrap_or_default(),
            value: if protected { String::new() } else { node.value().unwrap_or_default() },
            rect,
        });
        if !protected {
            stack.extend(node.filtered_children(common_filter).rev().map(|n| (n, id)));
        }
    }
    host::publish_accessibility(window, &items);
}
fn states(n: &NodeRef<'_>) -> u64 {
    // Bit positions are the air/OS accessible_state ABI in Accessible.h.
    let toggled = n.data().toggled();
    let flags = [
        (0, n.data().supports_action(Action::Focus)),
        (1, n.is_focused()),
        (2, n.is_disabled()),
        (3, toggled.is_some()),
        (4, toggled == Some(Toggled::True)),
        (5, toggled == Some(Toggled::Mixed)),
        (7, n.is_selectable()),
        (8, n.is_selected() == Some(true)),
        (9, n.is_multiselectable()),
        (10, n.data().is_expanded().is_some()),
        (11, n.data().is_expanded() == Some(true)),
        (12, n.is_text_input() && !n.is_read_only_or_disabled()),
        (13, n.is_text_input() && n.is_read_only()),
        (14, n.is_multiline()),
        (15, n.is_password_input()),
        (16, n.is_hidden()),
        (19, n.is_modal()),
        (21, n.is_required()),
        (23, n.is_busy()),
    ];
    flags.into_iter().filter(|(_, set)| *set).fold(0, |v, (bit, _)| v | (1 << bit))
}
