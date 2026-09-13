use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = PathBuf::from(&crate_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    #[cfg(target_os = "macos")]
    println!(
        "cargo:rustc-link-search=native={}",
        root_dir.join("libs/macos").display()
    );

    #[cfg(target_os = "windows")]
    println!(
        "cargo:rustc-link-search=native={}",
        root_dir.join("libs/windows").display()
    );

    #[cfg(target_os = "linux")]
    println!(
        "cargo:rustc-link-search=native={}",
        root_dir.join("libs/linux").display()
    );

    let config = cbindgen::Config::from_file("cbindgen.toml").unwrap_or_default();
    let out_dir = PathBuf::from(&crate_dir).join("include");
    let _ = std::fs::create_dir_all(&out_dir);

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .map(|bindings| {
            bindings.write_to_file(out_dir.join("fluyer_core.h"));
        })
        .ok();
}
