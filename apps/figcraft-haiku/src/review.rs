//! Review the bytes the builder will consume. Each page is rendered from the
//! same immutable snapshot. Automation can ask and poll; the native dialog decides.
use crate::host;
use figcraft_engine::{Session, cmd::file::MAX_DOCUMENT_FILE, file_access};
use figcraft_render::{Options, View};
use serde_json::{Value, json};
use std::sync::mpsc::{self, Receiver, TryRecvError};

struct Review {
    document: u64,
    revision: u64,
    path: String,
    digest: String,
    answer: Receiver<Result<bool, String>>,
    decision: Option<Result<bool, String>>,
}
#[derive(Default)]
pub struct Reviews(Option<Review>);

impl Reviews {
    pub fn handle(&mut self, session: &Session, digest: Option<&str>) -> Result<Value, String> {
        let ds = session.ds().map_err(|e| e.to_string())?;
        if let Some(asked) = digest {
            let r = self.0.as_mut().filter(|r| r.digest == asked).ok_or("Unknown review in this editor session")?;
            r.poll()?;
            let same = !ds.is_dirty() && ds.uid == r.document && ds.revision == r.revision && ds.path.as_ref() == Some(&r.path);
            let state = if !same {
                "stale"
            } else {
                match &r.decision {
                    None => "pending",
                    Some(Ok(false)) => "rejected",
                    Some(Ok(true)) => {
                        let bytes = file_access::read_capped(&r.path, MAX_DOCUMENT_FILE).map_err(|e| e.to_string())?;
                        if figcraft_format::image_hash(&bytes) == r.digest { "approved" } else { "stale" }
                    }
                    Some(Err(e)) => return Err(e.clone()),
                }
            };
            return Ok(json!({"status":state, "sha256":r.digest, "document":r.document, "revision":r.revision}));
        }
        let path = ds
            .path
            .as_deref()
            .filter(|p| p.to_ascii_lowercase().ends_with(".figcraft"))
            .ok_or("Save the design as a .figcraft file before requesting review")?;
        if ds.is_dirty() || ds.preview.is_some() || ds.branch_of.is_some() {
            return Err("Review the saved main design after saving or leaving the preview".into());
        }
        let bytes = file_access::read_capped(path, MAX_DOCUMENT_FILE).map_err(|e| e.to_string())?;
        let digest = figcraft_format::image_hash(&bytes);
        if let Some(r) = self.0.as_mut() {
            r.poll()?;
            if r.decision.is_none() {
                if r.document == ds.uid && r.revision == ds.revision && r.digest == digest && r.path == path {
                    return Ok(json!({"status":"pending", "sha256":digest, "document":ds.uid, "revision":ds.revision}));
                }
                return Err("Finish the open operator review before requesting another".into());
            }
        }
        let (tx, rx) = mpsc::channel();
        let name = std::path::Path::new(path).file_name().unwrap_or_default().to_string_lossy().into_owned();
        std::thread::Builder::new()
            .name("FigCraft design review".into())
            .stack_size(16 << 20)
            .spawn(move || {
                let _ = tx.send(review_pages(&bytes, &name));
            })
            .map_err(|e| e.to_string())?;
        self.0 = Some(Review { document: ds.uid, revision: ds.revision, path: path.into(), digest: digest.clone(), answer: rx, decision: None });
        Ok(json!({"status":"pending", "sha256":digest, "document":ds.uid, "revision":ds.revision}))
    }
}
impl Review {
    fn poll(&mut self) -> Result<(), String> {
        if self.decision.is_none() {
            self.decision = match self.answer.try_recv() {
                Ok(answer) => Some(answer),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Err("The review window stopped without a decision".into())),
            };
        }
        Ok(())
    }
}
fn review_pages(bytes: &[u8], name: &str) -> Result<bool, String> {
    let loaded = figcraft_format::load_bytes_with_versions(bytes).map_err(|e| e.to_string())?;
    if !loaded.issues.is_empty() || loaded.images_past_budget != 0 {
        return Err("Could not load the complete saved design for review".into());
    }
    let doc = loaded.doc;
    if doc.pages.is_empty() {
        return Err("The design has no pages".into());
    }
    let mut index = 0;
    loop {
        let page = doc.pages[index];
        let bounds = doc.page_content_bounds(page).unwrap_or_default();
        let scale = (1536.0 / bounds.width().max(bounds.height()).max(1.0)).min(2.0);
        let view = View {
            origin: (bounds.x0, bounds.y0),
            scale,
            width: (bounds.width() * scale).ceil().clamp(1.0, 1536.0) as u32,
            height: (bounds.height() * scale).ceil().clamp(1.0, 1536.0) as u32,
        };
        // A new empty page has no layers but still needs a visible page preview.
        let view = View { width: view.width.max(2), height: view.height.max(2), ..view };
        let options = Options::default();
        let pix = figcraft_render::render_view(&doc, page, &view, &options).ok_or("Could not render the review page")?;
        let page_name = &doc.node(page).ok_or("Missing review page")?.name;
        let title = format!("{name}   ·   {page_name}   ({}/{})", index + 1, doc.pages.len());
        let last = index + 1 == doc.pages.len();
        match host::review_page(&title, pix.data(), view.width, view.height, index > 0, last) {
            0 => return Ok(false),
            1 if index > 0 => index -= 1,
            2 if last => return Ok(true),
            2 => index += 1,
            _ => return Err("Review closed without a decision".into()),
        }
    }
}
