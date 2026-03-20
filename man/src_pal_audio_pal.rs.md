# src/pal/audio_pal.rs

## `pub fn play_samples( samples: Vec<f32>, sample_rate: u32, stop_flag: Arc<AtomicBool>, ) -> Result<(), String>`

*Line 41 · fn*

Play f32 samples through the default audio output device.
Resamples and channel-converts to match device capabilities.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=18" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
