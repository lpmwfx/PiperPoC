// TTS adapter — bridges Slint UI callbacks to core/gateway.

use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use slint::ComponentHandle;

use crate::AppWindow;
use crate::core::synth_core::discover_voices;
use crate::gateway::piper_ffi_gtw::Piper;
use crate::gateway::wav_gtw;
use crate::pal::audio_pal;

/// Wire all Slint callbacks for TTS functionality.
pub fn wire_callbacks(ui: &AppWindow, models_dir: PathBuf, espeak_data_path: PathBuf) {
    let voices = discover_voices(&models_dir);

    // Populate voice list in UI
    let voice_names: Vec<slint::SharedString> = voices
        .iter()
        .map(|v| v.name.as_str().into())
        .collect();
    // Rc: Slint model API requires ModelRc (shared ownership)
    let model = std::rc::Rc::new(slint::VecModel::from(voice_names));
    ui.global::<crate::AppState>().set_voices(model.into());

    // Shared voice model paths — read-only after init
    let voice_paths: Arc<Vec<PathBuf>> = Arc::new(
        voices.iter().map(|v| v.model_path.clone()).collect()
    );

    // Shared stop flag for cancelling playback
    let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    // -- Synthesize callback --
    let paths = Arc::clone(&voice_paths);
    let espeak = espeak_data_path.clone();
    let flag = Arc::clone(&stop_flag);
    let ui_weak = ui.as_weak();
    ui.global::<crate::AppState>().on_synthesize(move |text| {
        let text = text.to_string();
        // Read UI state on the UI thread (callback runs on UI thread)
        let ui_handle = ui_weak.unwrap();
        let idx = ui_handle.global::<crate::AppState>().get_selected_voice() as usize;
        let speed = ui_handle.global::<crate::AppState>().get_speed();

        let Some(model_path) = paths.get(idx).cloned() else { return; };
        let espeak_path = espeak.clone();

        flag.store(false, Ordering::Relaxed);
        let playback_stop = Arc::clone(&flag);

        ui_handle.global::<crate::AppState>().set_is_speaking(true);
        ui_handle.global::<crate::AppState>().set_status("Syntetiserer...".into());

        let weak = ui_weak.clone();
        std::thread::spawn(move || {
            let synth_result = (|| -> Result<(), String> {
                let piper = Piper::new(&model_path, &espeak_path)?;
                let mut opts = piper.default_options();
                opts.length_scale = speed as f32;

                let (samples, sample_rate) = piper.synthesize(&text, Some(&opts))?;
                audio_pal::play_samples(samples, sample_rate as u32, playback_stop)?;
                Ok(())
            })();

            let _ = weak.upgrade_in_event_loop(move |ui| {
                ui.global::<crate::AppState>().set_is_speaking(false);
                match synth_result {
                    Ok(()) => ui.global::<crate::AppState>().set_status("Klar".into()),
                    Err(e) => ui.global::<crate::AppState>().set_status(
                        format!("Fejl: {e}").into()
                    ),
                }
            });
        });
    });

    // -- Stop callback --
    let flag2 = Arc::clone(&stop_flag);
    ui.global::<crate::AppState>().on_stop(move || {
        flag2.store(true, Ordering::Relaxed);
    });

    // -- Save WAV callback --
    let paths2 = Arc::clone(&voice_paths);
    let espeak2 = espeak_data_path;
    let ui_weak2 = ui.as_weak();
    ui.global::<crate::AppState>().on_save_wav(move |text| {
        let text = text.to_string();
        let ui_handle = ui_weak2.unwrap();
        let idx = ui_handle.global::<crate::AppState>().get_selected_voice() as usize;
        let speed = ui_handle.global::<crate::AppState>().get_speed();

        let Some(model_path) = paths2.get(idx).cloned() else { return; };
        let espeak_path = espeak2.clone();

        ui_handle.global::<crate::AppState>().set_is_speaking(true);
        ui_handle.global::<crate::AppState>().set_status("Gemmer WAV...".into());

        let weak = ui_weak2.clone();
        std::thread::spawn(move || {
            let save_result = (|| -> Result<(), String> {
                let piper = Piper::new(&model_path, &espeak_path)?;
                let mut opts = piper.default_options();
                opts.length_scale = speed as f32;

                let (samples, sample_rate) = piper.synthesize(&text, Some(&opts))?;
                let wav_path = std::env::current_dir()
                    .unwrap_or_default()
                    .join("output.wav");
                wav_gtw::write_wav(&wav_path, &samples, sample_rate as u32)?;
                Ok(())
            })();

            let _ = weak.upgrade_in_event_loop(move |ui| {
                ui.global::<crate::AppState>().set_is_speaking(false);
                match save_result {
                    Ok(()) => ui.global::<crate::AppState>().set_status("WAV gemt!".into()),
                    Err(e) => ui.global::<crate::AppState>().set_status(
                        format!("Fejl: {e}").into()
                    ),
                }
            });
        });
    });
}
