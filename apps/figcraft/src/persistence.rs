use figcraft_engine::Prefs;
use figcraft_ui_egui::FigApp;
use serde_json::json;
use std::path::Path;

/// The preferences' file in the config folder.
pub(crate) const STATE: &str = "state.json";

/// Everything quitting keeps, all done when it returns (macOS then ends the process): the
/// preferences in `dir`, this run's recovery copies ([`Recovery::quit`](crate::recovery::Recovery::quit))
/// and the live sessions' ends (one this app hosts tells everyone and stops; a joined one leaves).
pub fn shutdown(app: &mut FigApp, dir: Option<&Path>, recovery: Option<crate::recovery::Recovery>) {
    if let Some(dir) = dir {
        write_prefs(dir, &app.session.prefs);
    }
    // Before the sessions end: a recovered online file's copy waits for its session's outcome.
    if let Some(r) = recovery {
        r.quit(&app.session, app.ui.quit);
    }
    drop(std::mem::take(&mut app.session.live));
}

/// Write `prefs` to `dir` atomically (a temporary file renamed over the old one: a crash leaves
/// either version whole). They hold online owner tokens, so on Unix only the owner can read them.
pub(crate) fn write_prefs(dir: &Path, prefs: &Prefs) {
    let tmp = dir.join("state.json.tmp");
    let written = std::fs::create_dir_all(dir)
        .and_then(|()| write_private(&tmp, json!({"prefs": prefs}).to_string().as_bytes()))
        .and_then(|()| std::fs::rename(&tmp, dir.join(STATE)));
    if let Err(e) = written {
        log::warn!("preferences: couldn't save them in {}: {e}", dir.display());
    }
}

/// Write `bytes` to `path`, owner read/write only on Unix from before the first byte (a leftover
/// file from a crash is narrowed too: the mode given at open only applies to new files).
pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
    let mut f = o.open(path)?;
    #[cfg(unix)]
    f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    f.write_all(bytes)
}
