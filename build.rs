/// Build script for PiperPoC.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // rustdocumenter disabled — bug #66: duplicates doc comments on every build
    // rulestools_documenter::document_project();
    rulestools_scanner::scan_project();

    // Dynamic piper + onnxruntime
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    println!("cargo:rustc-link-search=native={manifest_dir}/vendor/piper/lib");
    println!("cargo:rustc-link-lib=dylib=piper");

    slint_build::compile("ui/main.slint")?;
    Ok(())
}
