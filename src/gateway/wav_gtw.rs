// WAV file export via hound.

use std::path::Path;

/// Bits per sample for WAV output.
const BITS_PER_SAMPLE: u16 = 16;
/// Maximum amplitude for 16-bit signed integer.
const I16_MAX_AMPLITUDE: f32 = 32767.0;

/// Write f32 samples as a 16-bit WAV file.
pub fn write_wav(
    path: &Path,
    samples: &[f32],
    sample_rate: u32,
) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| format!("failed to create WAV: {e}"))?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_val = (clamped * I16_MAX_AMPLITUDE) as i16;
        writer.write_sample(i16_val)
            .map_err(|e| format!("failed to write sample: {e}"))?;
    }

    writer.finalize()
        .map_err(|e| format!("failed to finalize WAV: {e}"))?;

    Ok(())
}
