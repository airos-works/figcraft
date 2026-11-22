use crate::{host, persistence, recovery};
use figcraft_engine::{Prefs, Session};
use figcraft_ui_egui::{DialogFuture, Services};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::task::Poll;

pub fn config_dir() -> Option<PathBuf> {
    match std::env::var_os("FIGCRAFT_CONFIG_DIR") {
        Some(dir) => (!dir.is_empty()).then(|| PathBuf::from(dir)),
        None => host::settings_directory().map(|p| p.join("FigCraft")),
    }
}

type Answer = Rc<RefCell<Option<Option<PathBuf>>>>;
thread_local! {
    static PANEL: RefCell<(i32, Option<Answer>)> = const { RefCell::new((0, None)) };
}

fn pick(mode: i32, title: &str, name: &str, extensions: &[&str]) -> DialogFuture {
    let result = Answer::default();
    PANEL.with_borrow_mut(|(serial, outstanding)| {
        if let Some(old) = outstanding.replace(result.clone()) {
            *old.borrow_mut() = Some(None);
        }
        *serial = serial.wrapping_add(1);
        host::file_panel(*serial, mode, name, extensions, title);
    });
    Box::pin(std::future::poll_fn(move |_| match result.borrow_mut().take() {
        Some(path) => Poll::Ready(path),
        None => Poll::Pending,
    }))
}
pub fn answer(id: i32, ok: bool, path: &str) {
    PANEL.with_borrow_mut(|(serial, result)| {
        if id == *serial
            && let Some(r) = result.take()
        {
            *r.borrow_mut() = Some(ok.then(|| PathBuf::from(path)));
        }
    });
}

pub fn clipboard_text() -> Option<String> {
    clipboard_string("text/plain")
}
fn clipboard_string(mime: &str) -> Option<String> {
    host::clipboard_get(mime).map(|b| String::from_utf8_lossy(&b).trim_end_matches('\0').to_owned())
}

pub struct Native {
    review: crate::review::Reviews,
    recovery: recovery::Shared,
    ctx: egui::Context,
}
impl Native {
    pub fn new(recovery: recovery::Shared, ctx: egui::Context) -> Self {
        Self { recovery, ctx, review: Default::default() }
    }
}
impl Services for Native {
    fn native_design_preview(&mut self, session: &Session) -> Result<serde_json::Value, String> {
        let doc = session.doc().map_err(|e| e.to_string())?;
        let page = session.page().map_err(|e| e.to_string())?;
        let (native, composited) = figcraft_render::air_os::preview(doc, page)?;
        Ok(serde_json::json!({"page":page,"nativeControls":native,"compositedControls":composited}))
    }

    fn design_review(&mut self, session: &Session, digest: Option<&str>) -> Result<serde_json::Value, String> {
        self.review.handle(session, digest)
    }

    fn pick_open(&mut self, title: &str, exts: &[&str]) -> DialogFuture {
        pick(0, title, "", exts)
    }
    fn pick_save(&mut self, title: &str, name: &str, exts: &[&str]) -> DialogFuture {
        pick(1, title, name, exts)
    }
    fn pick_folder(&mut self, title: &str) -> DialogFuture {
        pick(2, title, "", &["*"])
    }
    fn clipboard_get_text(&mut self) -> Option<String> {
        clipboard_text()
    }
    fn clipboard_get_html(&mut self) -> Option<String> {
        clipboard_string("text/html")
    }
    fn clipboard_set_text(&mut self, s: &str) {
        host::clipboard_set(s, None);
    }
    fn clipboard_get_png(&mut self) -> Option<Vec<u8>> {
        host::clipboard_get("image/png")
    }
    fn clipboard_get_image(&mut self) -> Option<(u32, u32, Vec<u8>)> {
        let img = image::load_from_memory(&self.clipboard_get_png()?).ok()?.into_rgba8();
        Some((img.width(), img.height(), img.into_raw()))
    }
    fn clipboard_set_png(&mut self, w: u32, h: u32, rgba: &[u8]) {
        figcraft_ui_egui::egui_copy_png(&self.ctx, w, h, rgba);
    }
    fn open_url(&mut self, url: &str) {
        host::open_url(url);
    }
    fn reveal_path(&mut self, path: &Path) {
        host::reveal(&path.to_string_lossy());
    }
    fn system_dark(&self) -> Option<bool> {
        Some(host::dark())
    }
    fn system_language(&self) -> Option<String> {
        Some(host::locale().0)
    }
    fn save_prefs(&mut self, prefs: &Prefs) {
        if let Some(d) = config_dir() {
            persistence::write_prefs(&d, prefs);
        }
    }
    fn emergency_save(&mut self, s: &Session) {
        if let Some(r) = recovery::lock(&self.recovery).as_mut() {
            r.write(s, true);
            r.settle(s);
        }
    }
}
