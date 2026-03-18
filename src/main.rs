// PiperPoC — Slint GUI for testing piper1-gpl text-to-speech.
mod adapter;
mod core;
mod gateway;
mod pal;
mod shared;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = AppWindow::new()?;
    window.run()?;
    Ok(())
}
