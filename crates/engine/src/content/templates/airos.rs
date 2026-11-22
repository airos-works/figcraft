//! Editable air/OS Interface Kit components. Class and header metadata travel
//! with each main component in .figcraft files and the MCP inspector.
use super::kit::Kit;
use crate::Result;
use figcraft_doc::NodeId;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const PANEL: &str = "Panel";
const CONTROL: &str = "Control";
const PAPER: &str = "Document";
const TEXT: &str = "Panel text";
const EDGE: &str = "Navigation border";
const ACTIVE: &str = "Selection";
const TAB: &str = "Window tab";

struct Widget {
    class: &'static str,
    header: &'static str,
    label: &'static str,
    size: [f64; 2],
    look: Look,
}
#[derive(Clone, Copy)]
enum Look {
    Button,
    Icon,
    Check(bool),
    Radio(bool),
    Slider,
    Channels,
    Progress,
    Color,
    Field,
    Multiline,
    Text,
    PopUp,
    MenuBar,
    Menu,
    Item,
    Divider,
    List,
    Outline,
    Columns,
    Row,
    ScrollBar,
    Scroll,
    Tabs,
    Page,
    Split,
    Grid,
    Group,
    Box,
    Window,
    File(bool),
    Alert,
    Toolbar,
    Drag,
    Tip,
    Notification,
    Swatch,
    Spinner,
    Calendar,
    Date,
    Time,
    Shelf,
    Space,
    Print,
    Preview,
    Status,
    Activity,
}
macro_rules! widget {
    ($class:literal,$header:literal,$look:expr,$label:literal,$w:literal,$h:literal) => {
        Widget { class: $class, header: concat!($header, ".h"), look: $look, label: $label, size: [$w as f64, $h as f64] }
    };
}
use Look::*;
const GROUPS: &[(&str, &[Widget])] = &[
    (
        "Controls",
        &[
            widget!("BButton", "Button", Button, "Apply", 82, 26),
            widget!("BPictureButton", "PictureButton", Icon, "Picture", 28, 26),
            widget!("BCheckBox", "CheckBox", Check(false), "Show labels", 120, 22),
            widget!("BCheckBox / On", "CheckBox", Check(true), "Show labels", 120, 22),
            widget!("BRadioButton", "RadioButton", Radio(false), "Manual", 100, 22),
            widget!("BRadioButton / On", "RadioButton", Radio(true), "Automatic", 110, 22),
            widget!("BSlider", "Slider", Slider, "Volume", 155, 44),
            widget!("BChannelSlider", "ChannelSlider", Channels, "Level", 150, 54),
            widget!("BStatusBar", "StatusBar", Progress, "Copying", 155, 38),
            widget!("BColorControl", "ColorControl", Color, "Color", 320, 80),
            widget!("BTextControl", "TextControl", Field, "Name", 170, 25),
            widget!("BTextView", "TextView", Multiline, "Notes", 170, 64),
            widget!("BStringView", "StringView", Text, "Description", 120, 20),
            widget!("BOptionPopUp", "OptionPopUp", PopUp, "Automatic", 140, 25),
            widget!("BTextControl / Password", "TextControl", Field, "Password", 170, 25),
            widget!("BControl", "Control", Button, "Custom control", 125, 26),
            widget!("BChannelControl", "ChannelControl", Channels, "Channels", 150, 54),
            widget!("BOptionControl", "OptionControl", PopUp, "Option", 140, 25),
        ],
    ),
    (
        "Menus and collections",
        &[
            widget!("BMenuBar", "MenuBar", MenuBar, "File", 180, 24),
            widget!("BMenu", "Menu", Menu, "Open…", 140, 68),
            widget!("BPopUpMenu", "PopUpMenu", Menu, "Choose", 140, 68),
            widget!("BMenuItem", "MenuItem", Item, "Open…", 140, 24),
            widget!("BSeparatorItem", "SeparatorItem", Divider, "Separator", 142, 8),
            widget!("BMenuField", "MenuField", PopUp, "Format", 140, 25),
            widget!("BListView", "ListView", List, "Documents", 158, 62),
            widget!("BOutlineListView", "OutlineListView", Outline, "Project", 158, 62),
            widget!("BListItem", "ListItem", Row, "Item", 158, 20),
            widget!("BStringItem", "StringItem", Row, "Document", 158, 20),
            widget!("BScrollView", "ScrollView", Scroll, "Document", 158, 62),
            widget!("BScrollBar", "ScrollBar", ScrollBar, "Position", 144, 14),
            widget!("BColumnListView", "private/interface/ColumnListView", Columns, "Name", 172, 66),
            widget!("BRow", "private/interface/ColumnListView", Row, "Document", 170, 20),
            widget!("BColumn", "private/interface/ColumnListView", Item, "Name", 112, 24),
            widget!("BTabView", "TabView", Tabs, "General", 178, 64),
            widget!("BTab", "TabView", Button, "General", 82, 24),
        ],
    ),
    (
        "Column types",
        &[
            widget!("BField", "private/interface/ColumnListView", Row, "Field", 138, 20),
            widget!("GraphColumn", "private/interface/ColumnTypes", Progress, "Usage", 138, 40),
            widget!("BTitledColumn", "private/interface/ColumnTypes", Item, "Title", 116, 24),
            widget!("BStringColumn", "private/interface/ColumnTypes", Item, "Name", 116, 24),
            widget!("BDateColumn", "private/interface/ColumnTypes", Item, "Date modified", 140, 24),
            widget!("BSizeColumn", "private/interface/ColumnTypes", Item, "Size", 92, 24),
            widget!("BIntegerColumn", "private/interface/ColumnTypes", Item, "Count", 92, 24),
            widget!("BBitmapColumn", "private/interface/ColumnTypes", Item, "Preview", 112, 24),
            widget!("BStringField", "private/interface/ColumnTypes", Row, "Document", 138, 20),
            widget!("BDateField", "private/interface/ColumnTypes", Row, "22 Nov 2026", 138, 20),
            widget!("BSizeField", "private/interface/ColumnTypes", Row, "1.2 MiB", 110, 20),
            widget!("BIntegerField", "private/interface/ColumnTypes", Row, "12", 72, 20),
            widget!("BBitmapField", "private/interface/ColumnTypes", Icon, "Preview", 28, 26),
        ],
    ),
    (
        "Views and layouts",
        &[
            widget!("BView", "View", Page, "Content", 160, 64),
            widget!("BBox", "Box", Box, "Options", 160, 64),
            widget!("BGroupView", "GroupView", Group, "Group", 160, 52),
            widget!("BGroupLayout", "GroupLayout", Group, "Group", 160, 52),
            widget!("BGridView", "GridView", Grid, "Grid", 160, 64),
            widget!("BGridLayout", "GridLayout", Grid, "Grid", 160, 64),
            widget!("BCardView", "CardView", Page, "First card", 160, 64),
            widget!("BCardLayout", "CardLayout", Page, "First card", 160, 64),
            widget!("BSplitView", "SplitView", Split, "Sidebar", 168, 64),
            widget!("BSeparatorView", "SeparatorView", Divider, "Separator", 155, 6),
            widget!("BSpaceLayoutItem", "SpaceLayoutItem", Space, "Spacer", 64, 20),
            widget!("BDragger", "Dragger", Drag, "Replicant handle", 16, 16),
            widget!("BShelf", "Shelf", Shelf, "Replicants", 160, 56),
            widget!("BViewPort", "private/interface/ViewPort", Scroll, "Viewport", 158, 62),
            widget!("BTextToolTip", "private/interface/ToolTip", Tip, "Show details", 104, 24),
            widget!("BToolTip", "private/interface/ToolTip", Tip, "Show details", 104, 24),
        ],
    ),
    (
        "Windows and panels",
        &[
            widget!("BWindow", "Window", Window, "Preferences", 174, 76),
            widget!("BWindow / Document", "Window", Window, "Untitled", 174, 76),
            widget!("BWindow / Floating", "Window", Window, "Tools", 138, 76),
            widget!("BWindow / Modal", "Window", Alert, "Choose a setting", 174, 76),
            widget!("PromptWindow", "private/shared/PromptWindow", Field, "Name", 170, 25),
            widget!("BAlert", "Alert", Alert, "Save changes?", 174, 76),
            widget!("BFilePanel / Open", "FilePanel", File(false), "Open", 176, 78),
            widget!("BFilePanel / Save", "FilePanel", File(true), "Save", 176, 78),
            widget!("BPrintJob / Page setup", "PrintJob", Print, "Page setup", 176, 78),
            widget!("BPrintJob / Print", "PrintJob", Print, "Print", 176, 78),
            widget!("BNotification", "Notification", Notification, "Transfer complete", 176, 52),
            widget!("BDeskbar", "Deskbar", MenuBar, "Applications", 176, 24),
            widget!("BDirectWindow", "DirectWindow", Window, "Canvas", 174, 76),
            widget!("BWindowScreen", "WindowScreen", Page, "Full screen", 174, 76),
            widget!("BQuickLookPreview", "add-ons/quick_look/QuickLook", Preview, "Preview", 174, 76),
            widget!("BAboutWindow", "private/interface/AboutWindow", Alert, "About this app", 174, 76),
        ],
    ),
    (
        "Application kit views",
        &[
            widget!("BGLView", "opengl/GLView", Preview, "OpenGL view", 174, 76),
            widget!("BWebView", "WebView", Page, "Web content", 174, 76),
            widget!("BMediaTheme / Parameter panel", "MediaTheme", Channels, "Input levels", 150, 54),
            widget!("BMailSettingsView", "add-ons/mail_daemon/MailSettingsView", Field, "Mail account", 170, 25),
            widget!("BNetworkInterfaceListItem", "add-ons/network_settings/NetworkSettingsAddOn", Row, "Ethernet", 158, 20),
        ],
    ),
    (
        "Shared controls",
        &[
            widget!("BToolBar", "private/shared/ToolBar", Toolbar, "Tools", 158, 32),
            widget!("BIconButton", "private/shared/IconButton", Icon, "Tool", 28, 26),
            widget!("BBitmapButton", "private/shared/BitmapButton", Icon, "Bitmap", 28, 26),
            widget!("BAbstractSpinner", "private/interface/AbstractSpinner", Spinner, "8", 96, 24),
            widget!("SectionEdit", "private/shared/DateTimeEdit", Date, "01 / 01", 110, 25),
            widget!("BColorItem", "private/shared/ColorItem", Swatch, "Color", 92, 24),
            widget!("BSpinner", "private/interface/Spinner", Spinner, "8", 96, 24),
            widget!("BDecimalSpinner", "private/interface/DecimalSpinner", Spinner, "1.25", 96, 24),
            widget!("BCalendarView", "private/shared/CalendarView", Calendar, "November", 224, 168),
            widget!("DateEdit", "private/shared/DateTimeEdit", Date, "22 / 11 / 2026", 130, 25),
            widget!("TimeEdit", "private/shared/DateTimeEdit", Time, "09 : 30", 100, 25),
            widget!("BColorPreview", "private/shared/ColorPreview", Swatch, "Color", 48, 28),
            widget!("BColorMenuItem", "private/interface/ColorMenuItem", Swatch, "Color", 92, 24),
            widget!("BColorListView", "private/shared/ColorListView", Color, "Colors", 156, 54),
            widget!("BAboutMenuItem", "private/shared/AboutMenuItem", Item, "About…", 140, 24),
            widget!("BStripeView", "private/shared/StripeView", Activity, "Activity", 155, 18),
            widget!("BarberPole", "private/shared/BarberPole", Activity, "Working", 155, 18),
            widget!("IconView", "private/shared/IconView", Preview, "Icon", 52, 52),
            widget!("BView / Status line", "View", Status, "Ready", 158, 22),
        ],
    ),
];

impl Widget {
    fn native(&self) -> bool {
        figcraft_doc::air_os::has_native_widget(self.class.split(" / ").next().unwrap_or(self.class))
    }
}

pub(super) fn catalog() -> Value {
    Value::Array(
        GROUPS
            .iter()
            .flat_map(|(group, widgets)| {
                widgets.iter().map(move |w| {
                    json!({
                        "name":w.class, "class":w.class.split(" / ").next(), "header":w.header, "rendering": if w.native() { "nativeOnAirOS" } else { "designLayers" }, "group":group, "size":w.size,
                    })
                })
            })
            .collect(),
    )
}

fn paint(k: &mut Kit, parent: NodeId, name: &str, r: [f64; 4], fill: &str, edge: bool) -> Result<NodeId> {
    let mut props = json!({"fill":fill});
    if edge {
        props["stroke"] = json!(EDGE);
        props["strokeWeight"] = json!(1);
    }
    k.rect(parent, name, r, props)
}
fn label(k: &mut Kit, parent: NodeId, value: &str, x: f64, y: f64) -> Result<NodeId> {
    k.text_with(parent, value, "Control label", TEXT, json!({"x":x,"y":y}))
}
fn native_frame(k: &mut Kit, component: NodeId, w: &Widget, class: &str) -> Result<NodeId> {
    let [width, height] = w.size;
    let kind = if w.class.ends_with(" / Password") { w.class } else { class };
    let mut properties = Vec::new();
    if matches!(w.look, Button | Icon | Check(_) | Radio(_) | Field | Slider | Channels | Spinner | Date | Time | Color | PopUp)
        && !matches!(class, "BTab" | "BBitmapField" | "BColorListView")
    {
        properties.push(("Enabled", "boolean", json!(true)));
    }
    if let Check(on) | Radio(on) = w.look {
        properties.push(("Checked", "boolean", json!(on)));
    }
    let value = match w.look {
        Spinner => Some(w.label),
        Calendar | Date => Some("2026-11-22"),
        Time => Some("09:30:00"),
        Color | Swatch if matches!(class, "BColorControl" | "BColorPreview" | "BColorMenuItem" | "BColorItem") => Some("#3875AE"),
        Row if class == "BIntegerField" => Some("12"),
        Row if class == "BDateField" => Some("2026-11-22"),
        Row if class == "BSizeField" => Some("1258291"),
        Progress => Some("60"),
        Slider | Channels | ScrollBar => Some("40"),
        Field if w.class.ends_with("Password") => Some("sample"),
        Field => Some(""),
        PopUp if class == "BMenuField" => Some("Default"),
        _ => None,
    };
    if let Some(value) = value {
        properties.push(("Value", "text", json!(value)));
    }
    for (name, ty, default) in properties {
        k.run("component.addProperty", json!({"id":component.0,"name":name,"type":ty,"default":default}))?;
    }
    k.frame(
        component,
        "Native control",
        [0., 0., width, height],
        json!({
            "fills":[],"constraints":{"horizontal":"leftRight","vertical":"topBottom"},
            "pluginData":{"airOS.widget":kind,"airOS.label":w.label}
        }),
    )
}

fn face(k: &mut Kit, id: NodeId, w: &Widget) -> Result<Option<NodeId>> {
    let [width, height] = w.size;
    let mut caption = None;

    let fill = match w.look {
        Spinner | Date | Time | Row => PAPER,
        Tip => "#FFF6BF",
        Swatch => ACTIVE,
        _ => CONTROL,
    };
    paint(k, id, "Face", [0., 0., width, height], fill, true)?;
    if matches!(w.look, Icon) {
        k.line(id, "Pencil", "M7 18 L9 13 L18 4 L22 8 L13 17 Z", TEXT, 1.4)?;
    } else if !matches!(w.look, Swatch) {
        caption = Some(label(k, id, w.label, 8., (height - 16.) / 2.)?);
    }
    if matches!(w.look, PopUp | Spinner | Date | Time) {
        paint(k, id, "Arrow button", [width - 20., 0., 20., height], CONTROL, true)?;
        label(k, id, if matches!(w.look, Spinner) { "±" } else { "▾" }, width - 16., 3.)?;
    }
    Ok(caption)
}

fn track(k: &mut Kit, id: NodeId, w: &Widget) -> Result<Option<NodeId>> {
    let [width, _] = w.size;
    let mut caption = None;

    let top = if matches!(w.look, ScrollBar | Activity) { 0. } else { 24. };
    if top > 0. {
        caption = Some(label(k, id, w.label, 0., 0.)?);
    }
    let count = if matches!(w.look, Channels) { 2 } else { 1 };
    for i in 0..count {
        let y = top + f64::from(i) * 14.;
        let h = if matches!(w.look, Slider | Channels) { 5. } else { 12. };
        paint(k, id, "Track", [2., y, width - 4., h], CONTROL, true)?;
        if matches!(w.look, Slider | Channels | ScrollBar) {
            paint(k, id, "Thumb", [width * 0.40, y - 2., if matches!(w.look, ScrollBar) { 40. } else { 10. }, h + 4.], PANEL, true)?;
        } else {
            paint(k, id, "Complete", [3., y + 1., width * 0.6, h - 2.], ACTIVE, false)?;
        }
    }
    Ok(caption)
}

fn collection(k: &mut Kit, id: NodeId, w: &Widget) -> Result<Option<NodeId>> {
    let [width, height] = w.size;
    let mut caption = None;

    paint(k, id, "Background", [0., 0., width, height], if matches!(w.look, Menu | MenuBar | Toolbar) { PANEL } else { PAPER }, true)?;
    match w.look {
        MenuBar => {
            caption = Some(label(k, id, w.label, 6., 3.)?);
            label(k, id, "Edit", width * 0.48, 3.)?;
            label(k, id, "View", width * 0.72, 3.)?;
        }
        Toolbar => {
            for (i, icon) in ["+", "−", "↶", "↷"].iter().enumerate() {
                paint(k, id, "Tool", [3. + i as f64 * 32., 3., 29., 25.], CONTROL, true)?;
                label(k, id, icon, 12. + i as f64 * 32., 6.)?;
            }
        }
        Calendar => {
            caption = Some(label(k, id, w.label, 6., 3.)?);
            label(k, id, "M    T    W    T    F    S    S", 6., 24.)?;
            label(k, id, "1     2     3    4    5    6    7", 6., 44.)?;
        }
        _ => {
            if matches!(w.look, Columns) {
                paint(k, id, "Header", [0., 0., width, 20.], CONTROL, true)?;
                label(k, id, "Size", width * 0.68, 2.)?;
            } else {
                paint(k, id, "Selection", [1., 1., width - 2., 20.], "#C6DDF3", false)?;
            }
            caption = Some(label(k, id, w.label, 5., 2.)?);
            let sub = if matches!(w.look, Outline) { 18. } else { 5. };
            label(k, id, if matches!(w.look, Menu) { "Save" } else { "Item" }, sub, 24.)?;
            label(k, id, if matches!(w.look, Menu) { "Close" } else { "Details" }, sub, 44.)?;
            if matches!(w.look, Scroll) {
                paint(k, id, "Scroll bar", [width - 12., 0., 12., height], CONTROL, true)?;
                paint(k, id, "Thumb", [width - 11., 8., 10., 24.], PANEL, true)?;
            }
        }
    }
    Ok(caption)
}

fn container(k: &mut Kit, id: NodeId, w: &Widget) -> Result<Option<NodeId>> {
    let [width, height] = w.size;
    let mut caption = None;

    paint(k, id, "Panel", [0., 0., width, height], PANEL, true)?;
    match w.look {
        Tabs => {
            paint(k, id, "Active tab", [2., 1., 74., 23.], CONTROL, true)?;
            caption = Some(label(k, id, w.label, 8., 3.)?);
            label(k, id, "Advanced", 83., 3.)?;
        }
        Box => {
            caption = Some(label(k, id, w.label, 10., -6.)?);
        }
        Drag => {
            k.line(id, "Grip", "M3 13 L13 3 M7 13 L13 7 M11 13 L13 11", EDGE, 1.)?;
        }
        Split => {
            paint(k, id, "Sidebar", [1., 1., width * 0.42, height - 2.], PAPER, true)?;
            caption = Some(label(k, id, w.label, 5., 5.)?);
        }
        _ => {
            let rows = if matches!(w.look, Grid) { 2 } else { 1 };
            for j in 0..rows {
                for i in 0..2 {
                    let h = (height - 12.) / f64::from(rows);
                    paint(k, id, "Layout cell", [4. + f64::from(i) * width / 2., 4. + f64::from(j) * h, width / 2. - 8., h - 2.], PAPER, true)?;
                }
            }
        }
    }
    Ok(caption)
}

fn window_layers(k: &mut Kit, id: NodeId, w: &Widget) -> Result<Option<NodeId>> {
    let [width, height] = w.size;

    paint(k, id, "Content", [0., 18., width, height - 18.], PANEL, true)?;
    paint(k, id, "Title tab", [0., 0., width * 0.80, 19.], TAB, true)?;
    paint(k, id, "Close", [5., 5., 9., 9.], CONTROL, true)?;
    let caption = Some(label(k, id, w.label, 19., 1.)?);
    match w.look {
        File(save) => {
            paint(k, id, "File list", [5., 25., width - 10., 25.], PAPER, true)?;
            label(k, id, "Documents", 9., 28.)?;
            paint(k, id, "Action", [width - 58., 54., 52., 20.], CONTROL, true)?;
            label(k, id, if save { "Save" } else { "Open" }, width - 52., 55.)?;
        }
        Print => {
            label(k, id, "Paper", 7., 27.)?;
            paint(k, id, "Paper size", [59., 26., 95., 19.], PAPER, true)?;
            label(k, id, "A4", 64., 27.)?;
        }
        Alert => {
            label(k, id, "Cancel", 10., 48.)?;
            paint(k, id, "Action", [width - 57., 44., 49., 22.], CONTROL, true)?;
            label(k, id, "OK", width - 43., 47.)?;
        }
        Notification => {
            label(k, id, "Documents", 9., 27.)?;
        }
        Preview => {
            k.line(id, "Image", "M6 42 L18 28 L26 36 L32 30 L44 42 Z", EDGE, 1.)?;
        }
        _ => {}
    }
    Ok(caption)
}

fn component(k: &mut Kit, p: NodeId, w: &Widget, x: f64, y: f64) -> Result<NodeId> {
    let [width, height] = w.size;
    let class = w.class.split(" / ").next().unwrap_or(w.class);
    let id = k.node(
        "component",
        p,
        w.class,
        [x, y, width, height],
        json!({"fills":[],"clip":false,
        "description":format!("{class} · <{}>",w.header),"pluginData":{"airOS.class":class,"airOS.header":w.header}}),
    )?;
    let component = id;
    let id = if w.native() { native_frame(k, component, w, class)? } else { component };
    let mut caption = None;
    match w.look {
        Button | Icon | PopUp | Spinner | Date | Time | Item | Row | Status | Tip | Swatch => caption = face(k, id, w)?,
        Field => {
            caption = Some(label(k, id, w.label, 0., (height - 16.) / 2.)?);
            paint(k, id, "Editable text", [width * 0.50, 0., width * 0.50, height], PAPER, true)?;
            if w.class.ends_with("Password") {
                label(k, id, "••••••", width * 0.5 + 5., (height - 16.) / 2.)?;
            }
        }
        Check(on) | Radio(on) => {
            let r = [2., 4., 14., 14.];
            let style = json!({"fill":PAPER,"stroke":EDGE,"strokeWeight":1});
            if matches!(w.look, Radio(_)) {
                k.ellipse(id, "Indicator", r, style)?;
                if on {
                    k.ellipse(id, "Selected", [6., 8., 6., 6.], json!({"fill":ACTIVE}))?;
                }
            } else {
                k.rect(id, "Indicator", r, style)?;
                if on {
                    k.line(id, "Checked", "M4 11 L8 15 L14 6", TEXT, 2.)?;
                }
            }
            caption = Some(label(k, id, w.label, 23., 3.)?);
        }
        Slider | Channels | Progress | ScrollBar | Activity => caption = track(k, id, w)?,
        Color => {
            for (col, color) in ["#D85B56", "#EBA238", "#E4CA4C", "#76AA63", "#69B8B6", "#598CC7", "#8274BA", "#B86990"].iter().enumerate() {
                paint(k, id, "Color", [col as f64 * width / 8., 2., width / 8., 24.], color, true)?;
            }
            caption = Some(label(k, id, w.label, 2., 32.)?);
        }
        Multiline | Text | Page | Space => {
            if !matches!(w.look, Text | Space) {
                paint(k, id, "Background", [0., 0., width, height], PAPER, true)?;
            }
            if !matches!(w.look, Space) {
                caption = Some(label(k, id, w.label, 3., 2.)?);
            }
        }
        List | Outline | Columns | Scroll | Menu | MenuBar | Toolbar | Calendar => caption = collection(k, id, w)?,
        Divider => {
            paint(k, id, "Rule", [0., height / 2., width, 1.], EDGE, false)?;
        }
        Tabs | Box | Split | Group | Grid | Shelf | Drag => caption = container(k, id, w)?,
        Window | File(_) | Alert | Print | Notification | Preview => caption = window_layers(k, id, w)?,
    }
    if w.native()
        && (matches!(w.look, Spinner | Calendar | Date | Time)
            || matches!(class, "BColorControl" | "BIntegerField" | "BDateField" | "BSizeField" | "GraphColumn" | "BScrollBar"))
    {
        // These display a value, not a separate editable caption.
        caption = None;
    }
    if let Some(text) = caption {
        k.text_prop(component, "Label", w.label, &[text])?;
    }
    Ok(component)
}

pub(super) fn build(k: &mut Kit) -> Result<()> {
    k.colors(&[
        (PANEL, "#D8D8D8"),
        (CONTROL, "#EEEEEE"),
        (PAPER, "#FFFFFF"),
        (TEXT, "#202020"),
        (EDGE, "#777777"),
        (ACTIVE, "#3875AE"),
        (TAB, "#FFCE00"),
    ])?;
    k.type_styles(&[("Control label", 12., "Regular", 16.)])?;
    let design = k.page()?;
    k.rename_page("Design")?;
    let mut mains = BTreeMap::new();
    for (group, items) in GROUPS {
        let page = k.add_page(group)?;
        let mut y = 0.;
        for row in items.chunks(4) {
            let mut x = 0.;
            let mut row_height: f64 = 0.;
            for w in row {
                label(k, page, w.class, x, y)?;
                mains.insert(w.class, component(k, page, w, x, y + 25.)?);
                x += (w.size[0] + 28.).max(214.);
                row_height = row_height.max(w.size[1]);
            }
            y += (row_height + 50.).max(116.);
        }
    }
    k.go(design)?;
    let window = k.frame(design, "Document settings", [0., 0., 420., 202.], json!({"fills":[],"clip":false}))?;
    paint(k, window, "Title tab", [0., 0., 156., 22.], TAB, true)?;
    paint(k, window, "Close", [5., 6., 10., 10.], CONTROL, true)?;
    label(k, window, "Document settings", 22., 3.)?;
    paint(k, window, "Panel", [0., 22., 420., 180.], PANEL, true)?;
    let body = k.stack(window, "Settings", "vertical", 12., json!(12), json!({"x":6,"y":29,"crossAlign":"min"}))?;
    for (class, caption) in [("BTextControl", "Document name"), ("BCheckBox / On", "Show labels"), ("BButton", "Apply")] {
        k.instance(mains[class], body, json!({"Label":caption}), json!({}))?;
    }
    Ok(())
}
