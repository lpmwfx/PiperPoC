# src/gateway/piper_ffi_gtw.rs

## `pub struct PiperSynthesizer`

*Line 13 · struct*

Opaque C synthesizer handle.

---

## `pub struct PiperAudioChunk`

*Line 19 · struct*

Audio chunk from piper_synthesize_next.

---

## `pub struct PiperSynthesizeOptions`

*Line 34 · struct*

Synthesis options passed to piper.

---

## `pub struct Piper`

*Line 69 · struct*

Safe Rust wrapper around the piper C API.

---

## `pub fn new( model_path: &Path, espeak_data_path: &Path, ) -> Result<Self, String>`

*Line 79 · fn*

Create a new Piper synthesizer from model files.

---

## `pub fn default_options(&self) -> PiperSynthesizeOptions`

*Line 103 · fn*

Get default synthesis options for this voice.

---

## `pub fn synthesize( &self, text: &str, options: Option<&PiperSynthesizeOptions>, ) -> Result<(Vec<f32>, i32), String>`

*Line 109 · fn*

Synthesize text to a Vec of f32 samples. Returns (samples, sample_rate).

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=18" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
