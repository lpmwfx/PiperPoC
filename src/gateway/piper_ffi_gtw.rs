// FFI bindings for piper.h C API.

use std::ffi::{CString, c_char, c_float, c_int};
use std::path::Path;

/// Return code: success.
const PIPER_OK: c_int = 0;
/// Return code: synthesis complete.
const PIPER_DONE: c_int = 1;

/// Opaque C synthesizer handle.
#[repr(C)]
pub struct PiperSynthesizer {
    _private: [u8; 0],
}

/// Audio chunk from piper_synthesize_next.
#[repr(C)]
pub struct PiperAudioChunk {
    pub samples: *const c_float,
    pub num_samples: usize,
    pub sample_rate: c_int,
    pub is_last: bool,
    pub phonemes: *const u32,
    pub num_phonemes: usize,
    pub phoneme_ids: *const c_int,
    pub num_phoneme_ids: usize,
    pub alignments: *const c_int,
    pub num_alignments: usize,
}

/// Synthesis options passed to piper.
#[repr(C)]
pub struct PiperSynthesizeOptions {
    pub speaker_id: c_int,
    pub length_scale: c_float,
    pub noise_scale: c_float,
    pub noise_w_scale: c_float,
}

// SAFETY: These are C functions from libpiper — the extern block
// declares their signatures as defined in piper.h.
unsafe extern "C" {
    fn piper_create(
        model_path: *const c_char,
        config_path: *const c_char,
        espeak_data_path: *const c_char,
    ) -> *mut PiperSynthesizer;

    fn piper_free(synth: *mut PiperSynthesizer);

    fn piper_default_synthesize_options(
        synth: *mut PiperSynthesizer,
    ) -> PiperSynthesizeOptions;

    fn piper_synthesize_start(
        synth: *mut PiperSynthesizer,
        text: *const c_char,
        options: *const PiperSynthesizeOptions,
    ) -> c_int;

    fn piper_synthesize_next(
        synth: *mut PiperSynthesizer,
        chunk: *mut PiperAudioChunk,
    ) -> c_int;
}

/// Safe Rust wrapper around the piper C API.
pub struct Piper {
    synth: *mut PiperSynthesizer,
}

// SAFETY: Piper is only used from one thread at a time.
// The synthesizer is created, used for synthesis, then dropped on the same thread.
unsafe impl Send for Piper {}

impl Piper {
    /// Create a new Piper synthesizer from model files.
    pub fn new(
        model_path: &Path,
        espeak_data_path: &Path,
    ) -> Result<Self, String> {
        let model_str = model_path.to_str().ok_or("invalid model path")?;
        let espeak_str = espeak_data_path.to_str().ok_or("invalid espeak path")?;

        let model_c = CString::new(model_str).map_err(|e| e.to_string())?;
        let espeak_c = CString::new(espeak_str).map_err(|e| e.to_string())?;

        // SAFETY: Passing valid null-terminated C strings to piper_create.
        // config_path is null to use model_path + ".json" default.
        let synth = unsafe {
            piper_create(model_c.as_ptr(), std::ptr::null(), espeak_c.as_ptr())
        };

        if synth.is_null() {
            return Err("piper_create returned null".into());
        }

        Ok(Self { synth })
    }

    /// Get default synthesis options for this voice.
    pub fn default_options(&self) -> PiperSynthesizeOptions {
        // SAFETY: self.synth is valid (checked non-null in new()).
        unsafe { piper_default_synthesize_options(self.synth) }
    }

    /// Synthesize text to a Vec of f32 samples. Returns (samples, sample_rate).
    pub fn synthesize(
        &self,
        text: &str,
        options: Option<&PiperSynthesizeOptions>,
    ) -> Result<(Vec<f32>, i32), String> {
        let text_c = CString::new(text).map_err(|e| e.to_string())?;

        let opts_ptr = match options {
            Some(opts) => opts as *const PiperSynthesizeOptions,
            None => std::ptr::null(),
        };

        // SAFETY: self.synth is valid, text_c is a valid C string,
        // opts_ptr is either null or points to a valid options struct.
        let rc = unsafe {
            piper_synthesize_start(self.synth, text_c.as_ptr(), opts_ptr)
        };
        if rc != PIPER_OK {
            return Err(format!("piper_synthesize_start failed: {rc}"));
        }

        let mut all_samples = Vec::new();
        let mut sample_rate = 0i32;

        loop {
            let mut chunk = std::mem::MaybeUninit::<PiperAudioChunk>::zeroed();
            // SAFETY: self.synth is valid, chunk is a valid output pointer.
            // piper_synthesize_next fills chunk with audio data.
            let rc = unsafe {
                piper_synthesize_next(self.synth, chunk.as_mut_ptr())
            };

            if rc == PIPER_DONE {
                break;
            }
            if rc != PIPER_OK {
                return Err(format!("piper_synthesize_next failed: {rc}"));
            }

            // SAFETY: piper_synthesize_next returned PIPER_OK,
            // so chunk is fully initialized with valid data.
            let chunk = unsafe { chunk.assume_init() };
            sample_rate = chunk.sample_rate;

            if !chunk.samples.is_null() && chunk.num_samples > 0 {
                // SAFETY: samples pointer is valid for num_samples elements
                // as guaranteed by piper_synthesize_next contract.
                let slice = unsafe {
                    std::slice::from_raw_parts(chunk.samples, chunk.num_samples)
                };
                all_samples.extend_from_slice(slice);
            }
        }

        Ok((all_samples, sample_rate))
    }
}

impl Drop for Piper {
    fn drop(&mut self) {
        if !self.synth.is_null() {
            // SAFETY: self.synth was allocated by piper_create and is non-null.
            unsafe { piper_free(self.synth) };
        }
    }
}
