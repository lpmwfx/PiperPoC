// Audio playback via cpal.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::Duration;

/// Poll interval while waiting for audio playback to complete.
const PLAYBACK_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Resample mono samples from src_rate to dst_rate using linear interpolation.
fn resample(samples: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if src_rate == dst_rate {
        return samples.to_vec();
    }
    let ratio = src_rate as f64 / dst_rate as f64;
    let out_len = ((samples.len() as f64) / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + frac * (b - a));
    }
    out
}

/// Expand mono to stereo by duplicating each sample.
fn mono_to_stereo(samples: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        out.push(s);
        out.push(s);
    }
    out
}

/// Play f32 samples through the default audio output device.
/// Resamples and channel-converts to match device capabilities.
pub fn play_samples(
    samples: Vec<f32>,
    sample_rate: u32,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("no output audio device found")?;

    let default_config = device.default_output_config()
        .map_err(|e| format!("no default output config: {e}"))?;

    let device_rate = default_config.sample_rate().0;
    let device_channels = default_config.channels();

    // Resample if device rate differs from piper output
    let resampled = resample(&samples, sample_rate, device_rate);

    // Expand to stereo if device expects more than 1 channel
    let output_samples = if device_channels >= 2 {
        mono_to_stereo(&resampled)
    } else {
        resampled
    };

    let config = cpal::StreamConfig {
        channels: device_channels,
        sample_rate: cpal::SampleRate(device_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Arc: shared between audio callback thread and this polling thread
    let output = Arc::new(output_samples);
    let pos = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));

    // Arc: clones for the audio output callback closure
    let s = Arc::clone(&output);
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
