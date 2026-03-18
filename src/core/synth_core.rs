// Synthesis orchestration — coordinates piper and audio playback.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// ONNX model file extension.
const ONNX_EXT: &str = "onnx";

/// Voice definition — model path + display name.
pub struct Voice {
    pub name: String,
    pub model_path: PathBuf,
}

/// Discover available voices from a models directory.
/// Looks for `<dir>/<lang>/<name>.onnx` files.
pub fn discover_voices(models_dir: &Path) -> Vec<Voice> {
    let mut voices = Vec::new();

    let Ok(entries) = std::fs::read_dir(models_dir) else {
        return voices;
    };

    for lang_dir in entries.flatten() {
        if !lang_dir.file_type().is_ok_and(|ft| ft.is_dir()) {
            continue;
        }

        let Ok(model_files) = std::fs::read_dir(lang_dir.path()) else {
            continue;
        };

        for file in model_files.flatten() {
            let path = file.path();
            if path.extension() == Some(OsStr::new(ONNX_EXT)) {
                let name = path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                voices.push(Voice { name, model_path: path });
            }
        }
    }

    voices.sort_by(|a, b| a.name.cmp(&b.name));
    voices
}
