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

    // Build scripts run on the host, so `#[cfg(target_os)]` would pick the host's
    // libs when cross-compiling (Android from macOS). Read the target instead.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let lib_dir = match target_os.as_str() {
        "android" => {
            let abi = match env::var("CARGO_CFG_TARGET_ARCH")
                .unwrap_or_default()
                .as_str()
            {
                "aarch64" => "arm64-v8a",
                "arm" => "armeabi-v7a",
                "x86_64" => "x86_64",
                _ => "x86",
            };
            root_dir.join("libs/android").join(abi)
        }
        other => root_dir.join("libs").join(other),
    };
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

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
