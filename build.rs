/// Build script for PiperPoC.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // rustdocumenter disabled — bug #66: duplicates doc comments on every build
    // rulestools_documenter::document_project();
    rulestools_scanner::scan_project();

    slint_build::compile("ui/main.slint")?;
    Ok(())
}
