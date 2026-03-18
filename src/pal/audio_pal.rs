// Audio playback via cpal.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::Duration;

/// Poll interval while waiting for audio playback to complete.
const PLAYBACK_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Play f32 samples through the default audio output device.
pub fn play_samples(
    samples: Vec<f32>,
    sample_rate: u32,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("no output audio device found")?;

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Arc: shared between audio callback thread and this polling thread
    let samples = Arc::new(samples);
    let pos = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));

    // Arc: clones for the audio output callback closure
    let s = Arc::clone(&samples);
    let p = Arc::clone(&pos);
    let d = Arc::clone(&done);
    let sf = Arc::clone(&stop_flag);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            if sf.load(Ordering::Relaxed) {
                for sample in data.iter_mut() {
                    *sample = 0.0;
                }
                d.store(true, Ordering::Relaxed);
                return;
            }

            let current = p.load(Ordering::Relaxed);
            for (i, sample) in data.iter_mut().enumerate() {
                let idx = current + i;
                if idx < s.len() {
                    *sample = s[idx];
                } else {
                    *sample = 0.0;
                    d.store(true, Ordering::Relaxed);
                }
            }
            p.store(current + data.len(), Ordering::Relaxed);
        },
        |err| {
            eprintln!("audio stream error: {err}");
        },
        None,
    ).map_err(|e| format!("failed to build audio stream: {e}"))?;

    stream.play().map_err(|e| format!("failed to play stream: {e}"))?;

    while !done.load(Ordering::Relaxed) {
        std::thread::sleep(PLAYBACK_POLL_INTERVAL);
    }

    Ok(())
}
