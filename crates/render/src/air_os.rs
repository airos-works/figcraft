//! A native appearance is a frame inside an editable component. Its children form the
//! portable drawing. Siblings remain normal layers and can be used to decorate the widget.
use figcraft_doc::{Document, Node, NodeKind, PropDef};
use figcraft_geom::Affine;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tiny_skia::{FilterQuality, IntSize, Pixmap, PixmapPaint};

#[derive(PartialEq, Eq)]
struct Appearance {
    kind: String,
    label: String,
    value: String,
    enabled: bool,
    checked: bool,
    width: u32,
    height: u32,
}

impl Appearance {
    fn read(doc: &Document, frame: &Node, legacy_preview: bool) -> Option<Self> {
        let widget = frame.plugin_data.get("airOS.widget");
        let owner = if widget.is_some() {
            doc.node(frame.parent?)?
        } else if legacy_preview {
            frame
        } else {
            return None;
        };
        let (main, overrides) = match &owner.kind {
            NodeKind::Instance { component, props } => (doc.node(*component)?, Some(props)),
            NodeKind::Component { .. } => (owner, None),
            _ => return None,
        };
        let NodeKind::Component { props, .. } = &main.kind else { return None };
        let class = main.plugin_data.get("airOS.class")?;
        if !figcraft_doc::air_os::has_native_widget(class) {
            return None;
        }
        let kind = if let Some(kind) = widget {
            if kind != class && !(kind == "BTextControl / Password" && class == "BTextControl") {
                return None;
            }
            kind.as_str()
        } else {
            // The original template stored the class on the main and drew its direct
            // children. Keep those layers editable in the canvas and honor that metadata
            // in actual-size preview, including existing instance label overrides.
            if main.children.iter().any(|id| doc.node(*id).is_some_and(|n| n.plugin_data.contains_key("airOS.widget"))) {
                return None;
            }
            if class == "BTextControl" && main.name.ends_with(" / Password") { "BTextControl / Password" } else { class.as_str() }
        };
        let boolean = |name: &str, fallback| {
            overrides.and_then(|o| o.get(name)).and_then(|v| v.as_bool()).unwrap_or_else(|| match props.get(name) {
                Some(PropDef::Boolean { default }) => *default,
                _ => fallback,
            })
        };
        let value = overrides
            .and_then(|o| o.get("Value"))
            .and_then(|v| v.as_str())
            .or_else(|| match props.get("Value") {
                Some(PropDef::Text { default }) => Some(default.as_str()),
                _ => None,
            })
            .unwrap_or(match kind {
                "BSlider" | "BScrollBar" | "BChannelSlider" => "40",
                "BStatusBar" | "GraphColumn" => "60",
                "BTextControl / Password" => "sample",
                "BSizeField" => "1258291",
                "BIntegerField" => "12",
                "BSpinner" => "8",
                "BDecimalSpinner" => "1.25",
                "DateEdit" | "BCalendarView" => "2026-11-22",
                "TimeEdit" => "09:30",
                _ => "",
            });
        let label = figcraft_doc::air_os::widget_label(doc, frame)
            .or_else(|| overrides.and_then(|p| p.get("Label")).and_then(|v| v.as_str()))
            .or_else(|| frame.plugin_data.get("airOS.label").map(String::as_str))
            .unwrap_or("");
        let width = frame.width.round() as u32;
        let height = frame.height.round() as u32;
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || width * height > 1_048_576
            || kind.len() > 64
            || label.len() > 65536
            || value.len() > 65536
        {
            return None;
        }
        Some(Self {
            kind: kind.to_string(),
            label: label.into(),
            value: value.into(),
            enabled: boolean("Enabled", true),
            checked: boolean("Checked", main.name.ends_with(" / On")),
            width,
            height,
        })
    }
}

static CACHE: Mutex<VecDeque<(Appearance, Arc<Pixmap>)>> = Mutex::new(VecDeque::new());

pub fn clear_cache() {
    if let Ok(mut cache) = CACHE.lock() {
        cache.clear();
    }
}

pub(crate) fn draw(doc: &Document, pix: &mut Pixmap, frame: &Node, xf: Affine) -> Option<()> {
    let key = Appearance::read(doc, frame, false)?;
    let mut cache = CACHE.lock().ok()?;
    let image = if let Some(index) = cache.iter().position(|(item, _)| item == &key) {
        let entry = cache.remove(index)?;
        let image = Arc::clone(&entry.1);
        cache.push_front(entry);
        image
    } else {
        let bytes = figcraft_airos_controls::render(&key.kind, &key.label, &key.value, key.enabled, key.checked, key.width, key.height)?;
        let image = Arc::new(Pixmap::from_vec(bytes, IntSize::from_wh(key.width, key.height)?)?);
        cache.push_front((key, Arc::clone(&image)));
        let mut size = 0;
        cache.truncate(128);
        cache.retain(|(_, pix)| {
            size += pix.data().len();
            size <= 16 << 20
        });
        image
    };
    drop(cache);
    let fit = Affine::scale_non_uniform(frame.width / f64::from(image.width()), frame.height / f64::from(image.height()));
    pix.draw_pixmap(
        0,
        0,
        image.as_ref().as_ref(),
        &PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() },
        crate::draw::ts_transform(xf * fit),
        None,
    );
    Some(())
}

/// Show the active design at one Interface Kit unit per design pixel. Leave transformed,
/// clipped or overpainted widgets in the composited image so their stacking stays intact.
pub fn preview(doc: &Document, page: figcraft_doc::NodeId) -> Result<(usize, usize), String> {
    let bounds = doc.page_content_bounds(page).ok_or("Place a control on this page before previewing it")?.expand();
    let (w, h) = (bounds.width() as u32, bounds.height() as u32);
    if w == 0 || h == 0 || w > 4096 || h > 4096 || w * h > 4_194_304 {
        return Err("Actual-size preview supports pages up to 4096 units per side and 4 megapixels".into());
    }
    let mut widgets = Vec::new();
    let mut total = 0;
    for id in doc.subtree(page) {
        let Some(n) = doc.node(id) else { continue };
        let Some(appearance) = Appearance::read(doc, n, true) else { continue };
        total += 1;
        if total > 512 {
            return Err("Actual-size preview supports up to 512 native controls per page".into());
        }
        let xf = doc.world_transform(id).as_coeffs();
        if xf[..4] != [1., 0., 0., 1.] {
            continue;
        }
        let Some(area) = doc.world_bounds(id) else { continue };
        let mut current = n;
        let mut visible = true;
        while let Some(parent) = current.parent.and_then(|p| doc.node(p)) {
            if !current.visible
                || current.is_mask
                || !matches!(current.blend, figcraft_color::BlendMode::Normal | figcraft_color::BlendMode::PassThrough)
                || current.opacity != 1.
                || !current.effects.is_empty()
                || parent.hides_contents()
                || !parent.strokes.is_empty()
                || (parent.clip && doc.world_bounds(parent.id).is_some_and(|clip| clip.intersect(area) != area))
            {
                visible = false;
                break;
            }
            let mut above = false;
            for sibling_id in parent.paint_order() {
                if sibling_id == current.id {
                    above = true;
                    continue;
                }
                let Some(sibling) = doc.node(sibling_id) else { continue };
                if sibling.visible
                    && (above || sibling.is_mask)
                    && crate::paint_bounds(doc, sibling_id).is_some_and(|rect| rect.intersect(area).area() > 0.)
                {
                    visible = false;
                    break;
                }
            }
            if !visible {
                break;
            }
            current = parent;
        }
        if visible {
            widgets.push(figcraft_airos_controls::Widget {
                kind: appearance.kind,
                label: appearance.label,
                value: appearance.value,
                enabled: appearance.enabled,
                checked: appearance.checked,
                width: appearance.width,
                height: appearance.height,
                x: (xf[4] - bounds.x0).round() as i32,
                y: (xf[5] - bounds.y0).round() as i32,
            });
        }
    }
    let view = crate::View { origin: (bounds.x0, bounds.y0), scale: 1., width: w, height: h };
    let pixels = crate::render_view(doc, page, &view, &crate::Options::default()).ok_or("Could not render the current design page")?;
    let title = format!("{} · {} — air/OS preview", doc.name, doc.node(page).ok_or("Missing design page")?.name);
    let placed = figcraft_airos_controls::present(&title, w, h, pixels.data(), widgets).ok_or("Could not open the native design preview")?;
    Ok((placed, total - placed))
}
