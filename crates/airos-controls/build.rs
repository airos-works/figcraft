use std::{path::PathBuf, process::Command};

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("haiku") {
        return;
    }
    println!("cargo:rerun-if-changed=controls.cpp");
    println!("cargo:rerun-if-changed=preview.h");
    let mut native = cc::Build::new();
    native.cpp(true).std("c++17").file("controls.cpp").flag("-Wno-multichar");
    let compiler = native.get_compiler();
    let sysroot = Command::new(compiler.path()).args(compiler.args()).arg("-print-sysroot").output().expect("query air/OS SDK sysroot");
    assert!(sysroot.status.success(), "query air/OS SDK sysroot");
    let root = String::from_utf8_lossy(&sysroot.stdout);
    let develop = PathBuf::from(if root.trim().is_empty() { "/" } else { root.trim() }).join("boot/system/develop");
    let headers = develop.join("headers/private");
    println!("cargo:rustc-link-search=native={}", develop.join("lib").display());
    native.include(headers.join("interface")).include(headers.join("shared")).compile("figcraft_controls");
    for library in ["static=columnlistview", "static=shared", "static=localestub", "be", "translation", "tracker"] {
        println!("cargo:rustc-link-lib={library}");
    }
}
