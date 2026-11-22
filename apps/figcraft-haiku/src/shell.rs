use crate::{
    args::Args,
    control_server,
    host::{self, RawEvent, kind},
    persistence,
    platform::Haiku,
    recovery, services,
    viewports::{Viewports, WindowInput},
};
use egui::{ViewportCommand, ViewportId};
use figcraft_engine::{Session, file_access::Roots};
use figcraft_ui_egui::{
    FigApp, Services,
    windows::{self, with_app},
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn run(root: i32, scale: f32, args: Args, roots: Option<Arc<Roots>>) -> Result<(), String> {
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(4);
    let _ = rayon::ThreadPoolBuilder::new().num_threads(workers).build_global();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let dev = host::device_scale();
    let views = Viewports::new(&ctx, Haiku::new()?, root, scale * dev, dev);
    let fonts = ctx.clone();
    std::thread::spawn(move || {
        figcraft_text::scan_system_fonts();
        fonts.request_repaint();
    });
    let copies = recovery::Shared::default();
    let dir = services::config_dir();
    let native = services::Native::new(copies.clone(), ctx.clone());
    let mut session = Session::empty();
    let wake = ctx.clone();
    session.live.install(Box::new(figcraft_collab_net::NetLink::new(Arc::new(move || wake.request_repaint()))));
    let prefs = dir
        .as_ref()
        .and_then(|d| std::fs::read(d.join(persistence::STATE)).ok())
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| serde_json::from_value(v.get("prefs")?.clone()).ok());
    let first = prefs.is_none();
    if let Some(p) = prefs {
        session.prefs = p;
    }
    session.plugins.install_dir = dir.as_ref().map(|d| d.join("Plugins"));
    let mut open_error = None;
    for path in args.files {
        if let Err(e) = session.execute("file.open", &json!({"path":path})) {
            open_error = Some(format!("{path}: {e}"));
        }
    }
    let recovery_dir = dir.as_ref().map(|d| d.join("Recovery"));
    let recovered = recovery_dir.as_deref().map(|p| recovery::recover_all(p, &mut session)).unwrap_or_default();
    if let Some(mut r) = recovery_dir.as_deref().and_then(recovery::Recovery::start) {
        r.adopt(recovered.waiting);
        *recovery::lock(&copies) = Some(r);
    }
    if session.docs.is_empty() && (args.control.is_some() || (!first && !session.prefs.start_on_home)) {
        session.execute("file.new", &json!({})).map_err(|e| e.to_string())?;
    }
    figcraft_ui_egui::i18n::set(&session.prefs.language, native.system_language().as_deref());
    let mut fig = FigApp::new(session, Box::new(native));
    fig.automation_roots = roots;
    fig.ui.home = false;
    fig.ui.font_checks = fig.session.docs.iter().map(|d| d.uid).collect();
    fig.note_recovered(recovered.copies);
    if let Some(e) = open_error {
        fig.show_error(e);
    }
    if let Some(link) = &args.join {
        figcraft_ui_egui::live::open_invite(&mut fig, link);
    }
    let queue = Arc::new(Mutex::new(Vec::new()));
    let context = Arc::new(Mutex::new(Some(ctx.clone())));
    let mut serving = args.control;
    if let Some(port) = serving {
        control_server::try_start(port, Duration::ZERO, queue.clone(), context.clone(), fig.session.events.clone()).map_err(|e| e.to_string())?;
    }
    windows::install(fig);
    let mut title = String::new();
    'running: loop {
        host::wait(views.timeout());
        while let Some((ev, text)) = host::next_event() {
            if let Some(input) = window_input(&ev, &text) {
                views.input(ev.window, input);
                continue;
            }
            match ev.kind {
                kind::QUIT_REQUESTED => {
                    views.input(root, WindowInput::Close);
                }
                kind::DIALOG_ANSWER => services::answer(ev.id, ev.x != 0.0, &text),
                kind::START_CONTROL => {
                    let result = with_app(|app| -> Result<(), String> {
                        let (port, read, write, files): (u16, Vec<String>, Vec<String>, Vec<String>) =
                            serde_json::from_str(&text).map_err(|e| e.to_string())?;
                        if port == 0 || read.is_empty() || write.is_empty() {
                            return Err("Specify a port and both project roots".into());
                        }
                        let roots = Arc::new(Roots::new(&read, &write)?);
                        if let Some(current) = serving {
                            if current != port || !app.automation_roots.as_ref().is_some_and(|r| r.to_json() == roots.to_json()) {
                                return Err("This editor already serves another project or port".into());
                            }
                        } else {
                            control_server::try_start(port, Duration::ZERO, queue.clone(), context.clone(), app.session.events.clone())
                                .map_err(|e| e.to_string())?;
                            serving = Some(port);
                            app.automation_roots = Some(roots.clone());
                        }
                        figcraft_engine::file_access::confined(Some(&roots), || -> Result<(), String> {
                            for path in files {
                                app.session.execute("file.open", &json!({"path":path})).map_err(|e| e.to_string())?;
                                app.ui.home = false;
                            }
                            Ok(())
                        })
                    })
                    .unwrap_or_else(|| Err("Editor is not available".into()));
                    host::control_answer(&ev, result.as_ref().err().map_or("", String::as_str));
                }
                kind::OPEN_FILES => {
                    with_app(|app| {
                        for path in text.lines() {
                            if let Err(e) = app.session.execute("file.open", &json!({"path":path})) {
                                app.show_error(format!("{path}: {e}"));
                            } else {
                                app.ui.home = false;
                            }
                        }
                    });
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                }
                kind::COLORS => {
                    figcraft_render::air_os::clear_cache();
                    // Canvas jobs and retained tiles watch the font/layout generation.
                    figcraft_text::clear_layout_cache();
                    let dev = host::device_scale();
                    views.set_density(host::ui_scale() * dev, dev);
                }
                _ => continue,
            }
            ctx.request_repaint();
        }
        for id in views.take_due() {
            if id != ViewportId::ROOT {
                if views.run_deferred(id).close_root {
                    break 'running;
                }
                continue;
            }
            let Some(mut raw) = views.root_input() else { continue };
            with_app(|app| raw.events.extend(windows::take_input(app, id)));
            let closing = raw.viewport().close_requested();
            let out = ctx.run_ui(raw, |ui| {
                with_app(|app| {
                    control_server::drain_queue(&queue, &mut app.control_queue);
                    windows::guard(app, &ctx, |app| app.background_tick(&ctx));
                    windows::on_close_request(app, &ctx);
                    let next = app.title();
                    if title != next {
                        title = next;
                        ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
                    }
                    if let Some(wait) = recovery::lock(&copies).as_mut().and_then(|r| r.tick(&app.session, app.now_secs())) {
                        ctx.request_repaint_after(Duration::from_secs_f64(wait));
                    }
                    windows::guard(app, &ctx, |app| app.ui(ui));
                });
            });
            let cancel = out.viewport_output.get(&id).is_some_and(|v| v.commands.contains(&ViewportCommand::CancelClose));
            if views.finish(id, out).close_root || (closing && !cancel) {
                break 'running;
            }
        }
        views.chores();
    }
    with_app(|app| persistence::shutdown(app, dir.as_deref(), recovery::lock(&copies).take()));
    Ok(())
}
fn window_input<'a>(ev: &RawEvent, text: &'a str) -> Option<WindowInput<'a>> {
    Some(match ev.kind {
        kind::ACCESS_ACTION => WindowInput::Accessible(crate::accessibility::request(ev.id, ev.node)?),
        kind::MOUSE_MOVED | kind::MOUSE_DOWN | kind::MOUSE_UP => {
            WindowInput::Pointer { x: ev.x, y: ev.y, buttons: ev.buttons, modifiers: ev.modifiers }
        }
        kind::POINTER_GONE => WindowInput::PointerGone,
        // A wheel's whole steps, or (`repeat` 1) a touchpad's scroll gesture with its unrounded
        // steps and its phase in `clicks` (`host.cpp`).
        kind::WHEEL => WindowInput::Wheel { dx: ev.dx, dy: ev.dy, modifiers: ev.modifiers, gesture: (ev.repeat != 0).then_some(ev.clicks) },
        kind::KEY_DOWN | kind::KEY_UP => {
            WindowInput::Key { pressed: ev.kind == kind::KEY_DOWN, code: ev.key, raw_char: ev.raw_char, modifiers: ev.modifiers, text }
        }
        kind::MODIFIERS => WindowInput::Modifiers(host::modifiers()),
        kind::FOCUS => WindowInput::Focus(ev.x != 0.0),
        kind::CLOSE => WindowInput::Close,
        // Files dropped on a window go to the interface as the desktop app's do: on the canvas
        // they are placed, elsewhere opened.
        kind::DROPPED_FILES => WindowInput::Dropped { x: ev.x, y: ev.y, paths: text.lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect() },
        // The next frame picks up the new size, position or state.
        kind::RESIZED | kind::MOVED | kind::MINIMIZED => WindowInput::Changed,
        // An input method's composition: `id` says what, the pre-edit's selected clause is
        // bytes `key` to `raw_char` (-1: none).
        kind::IME => match ev.id {
            host::ime::PREEDIT => WindowInput::Preedit { text, selection: (ev.key >= 0 && ev.raw_char >= 0).then_some((ev.key, ev.raw_char)) },
            host::ime::COMMIT => WindowInput::Commit(text),
            host::ime::STOPPED => WindowInput::ImeStopped,
            _ => return None,
        },
        _ => return None,
    })
}
