// PiperPoC — Slint GUI for testing piper1-gpl text-to-speech.
mod adapter;
mod core;
mod gateway;
mod pal;
mod shared;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    // Look for models relative to exe, then fall back to cwd
    let models_dir = if exe_dir.join("models").exists() {
        exe_dir.join("models")
    } else {
        std::path::PathBuf::from("models")
    };

    let espeak_data = if exe_dir.join("vendor/piper/espeak-ng-data").exists() {
        exe_dir.join("vendor/piper/espeak-ng-data")
    } else {
        std::path::PathBuf::from("vendor/piper/espeak-ng-data")
    };

    let window = AppWindow::new()?;
    adapter::tts_adp::wire_callbacks(&window, models_dir, espeak_data);
    window.run()?;
    Ok(())
}
