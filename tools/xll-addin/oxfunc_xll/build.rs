use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=EXCEL_XLL_SDK_DIR");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let default_sdk = manifest_dir
        .join("..")
        .join("..")
        .join("..")
        .join(".tmp")
        .join("excelxllsdk_extracted")
        .join("2013 Office System Developer Resources")
        .join("Excel2013XLLSDK");

    let sdk_root = env::var("EXCEL_XLL_SDK_DIR")
        .map(PathBuf::from)
        .unwrap_or(default_sdk);
    let include_dir = sdk_root.join("INCLUDE");
    let xlcall_cpp = sdk_root.join("SRC").join("XLCALL.CPP");
    let bridge_cpp = manifest_dir.join("native").join("registration_bridge.cpp");

    if !include_dir.exists() || !xlcall_cpp.exists() || !bridge_cpp.exists() {
        panic!(
            "Excel XLL SDK or bridge source not found. Ensure these exist:\n  {}\n  {}\n  {}",
            include_dir.display(),
            xlcall_cpp.display(),
            bridge_cpp.display()
        );
    }

    println!("cargo:rerun-if-changed={}", xlcall_cpp.display());
    println!("cargo:rerun-if-changed={}", bridge_cpp.display());
    cc::Build::new()
        .cpp(true)
        .file(xlcall_cpp)
        .file(bridge_cpp)
        .include(include_dir)
        .compile("oxfunc_xll_bridge");
}
