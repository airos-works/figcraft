#![recursion_limit = "256"]
#![deny(unsafe_code)]

#[cfg(target_os = "haiku")]
#[path = "../../figcraft/src/args.rs"]
mod args;

#[cfg(target_os = "haiku")]
#[path = "../../figcraft/src/control_server.rs"]
mod control_server;

#[cfg(target_os = "haiku")]
#[path = "../../figcraft/src/recovery.rs"]
mod recovery;

#[cfg(target_os = "haiku")]
#[path = "../../figcraft/src/persistence.rs"]
mod persistence;

#[cfg(target_os = "haiku")]
mod damage;

#[cfg(target_os = "haiku")]
mod keys;

#[cfg(target_os = "haiku")]
mod raster;

#[cfg(target_os = "haiku")]
mod texture_fit;

#[cfg(target_os = "haiku")]
mod viewports;

#[cfg(target_os = "haiku")]
mod gpu;

#[cfg(target_os = "haiku")]
mod host;

#[cfg(target_os = "haiku")]
mod platform;

#[cfg(target_os = "haiku")]
mod present;

#[cfg(target_os = "haiku")]
mod scanout;

#[cfg(target_os = "haiku")]
mod accessibility;
#[cfg(target_os = "haiku")]
mod review;
#[cfg(target_os = "haiku")]
mod services;

#[cfg(target_os = "haiku")]
mod shell;

#[cfg(not(target_os = "haiku"))]
fn main() {
    eprintln!("The air/OS host requires Haiku. Use the figcraft desktop binary on this platform.");
    std::process::exit(1);
}

#[cfg(target_os = "haiku")]
fn main() -> std::process::ExitCode {
    env_logger::init();
    if let Err(e) = run() {
        eprintln!("FigCraft: {e}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

#[cfg(target_os = "haiku")]
fn run() -> Result<(), String> {
    use std::sync::Arc;
    let argv: Vec<_> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "--version" || a == "-V") {
        println!("FigCraft {} for air/OS", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let mut args = args::parse(&argv);
    if let Some(flag) = args.unknown.first() {
        return Err(format!("unknown option {flag}"));
    }
    args.control = args.control.or_else(|| std::env::var("FIGCRAFT_CONTROL_PORT").ok()?.parse().ok());
    let roots = if args.read_roots.is_empty() && args.write_roots.is_empty() {
        None
    } else {
        Some(Arc::new(figcraft_engine::file_access::Roots::new(&args.read_roots, &args.write_roots).map_err(|e| e.to_string())?))
    };
    // Validate the same root policy before listening as the headless MCP server.
    if args.control.is_some() && (args.read_roots.is_empty() || args.write_roots.is_empty()) {
        return Err("--control requires --automation-read-root DIR and --automation-write-root DIR".into());
    }
    host::export_locale();
    let signature = "application/x-vnd.airos-FigCraft";
    if host::init(signature)? == host::Launch::HandedOver {
        if let Some(port) = args.control {
            let absolute = |paths: &[String]| -> Result<Vec<String>, String> {
                paths.iter().map(|p| std::path::absolute(p).map(|p| p.to_string_lossy().into_owned()).map_err(|e| e.to_string())).collect()
            };
            let request = serde_json::to_string(&(port, absolute(&args.read_roots)?, absolute(&args.write_roots)?, absolute(&args.files)?))
                .map_err(|e| e.to_string())?;
            host::start_control(signature, &request)?;
        }
        return Ok(());
    }
    host::claim_types(signature, &["application/x-figcraft"]);
    let scale = host::ui_scale();
    let root = host::open_window(&host::WindowSpec {
        title: "FigCraft".into(),
        position: None,
        size: [1120.0 * scale, 720.0 * scale],
        min_size: Some([640.0 * scale, 420.0 * scale]),
        max_size: None,
        decorations: true,
        resizable: true,
        visible: true,
        zoomed: false,
        fullscreen: false,
        floating: false,
        buttons: [true; 3],
    })
    .ok_or("could not open the editor window")?;
    let worker = std::thread::Builder::new()
        .name("FigCraft interface".into())
        .stack_size(32 << 20)
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| shell::run(root, scale, args, roots)))
                .unwrap_or_else(|_| Err("the editor stopped unexpectedly".into()));
            if let Err(e) = &result {
                host::alert("FigCraft", e);
            }
            host::quit();
            result
        })
        .map_err(|e| e.to_string())?;
    host::run();
    worker.join().map_err(|_| "interface thread failed")?
}
