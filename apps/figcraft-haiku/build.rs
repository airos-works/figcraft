fn main() {
    println!("cargo:rerun-if-changed=native/host.cpp");
    println!("cargo:rerun-if-changed=native/accessibility.h");
    println!("cargo:rerun-if-changed=native/review.cpp");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("haiku") {
        return;
    }
    cc::Build::new()
        .cpp(true)
        .file("native/host.cpp")
        .file("native/review.cpp")
        .flag("-Wno-multichar")
        .flag("-Wno-unused-parameter")
        .opt_level(2)
        .compile("figcraft_haiku_host");
    // game: BDirectWindow, which lets frames go straight into the screen.
    for lib in ["be", "game", "tracker", "translation", "stdc++"] {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }
}
