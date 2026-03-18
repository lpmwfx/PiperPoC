# src/pal/audio_pal.rs

## `pub fn play_samples( samples: Vec<f32>, sample_rate: u32, stop_flag: Arc<AtomicBool>, ) -> Result<(), String>`

*Line 41 · fn*

Play f32 samples through the default audio output device.
Resamples and channel-converts to match device capabilities.

---

