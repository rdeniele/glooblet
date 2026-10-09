//! Non-blocking sound playback and upload validation for Click Studio.
//!
//! Playback runs on a dedicated worker thread that owns the single rodio
//! `OutputStream` for the process. Callers (Tauri commands, the mouse hook)
//! only ever send a lightweight message down a channel — they never touch
//! the audio device directly, so a slow or failing audio backend can't block
//! the UI thread or the click-hook callback.

pub mod hook;

use crate::error::{AppError, AppResult};
use rodio::Source;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

pub const MAX_UPLOAD_BYTES: u64 = 5 * 1024 * 1024; // 5 MB
pub const MAX_DURATION_MS: u32 = 8_000; // 8 seconds
pub const ALLOWED_EXTENSIONS: &[&str] = &["wav", "mp3", "ogg"];

enum AudioCommand {
    Play { path: PathBuf, volume: f32 },
    StopAll,
}

pub struct AudioEngine {
    tx: Sender<AudioCommand>,
}

impl AudioEngine {
    /// Spawns the worker thread and opens the default output device. If no
    /// audio device is available, playback calls become silent no-ops rather
    /// than erroring the whole app — Click Studio just reports "unavailable".
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<AudioCommand>();

        std::thread::spawn(move || {
            let stream = rodio::OutputStream::try_default();
            let (_stream, handle) = match stream {
                Ok((s, h)) => (Some(s), Some(h)),
                Err(e) => {
                    log::warn!("no audio output device available: {e}");
                    (None, None)
                }
            };

            let mut active_sinks: Vec<rodio::Sink> = Vec::new();

            while let Ok(cmd) = rx.recv() {
                // Drop sinks that finished playing so the Vec doesn't grow forever.
                active_sinks.retain(|s| !s.empty());

                match cmd {
                    AudioCommand::Play { path, volume } => {
                        let Some(handle) = handle.as_ref() else { continue };
                        let file = match std::fs::File::open(&path) {
                            Ok(f) => f,
                            Err(e) => {
                                log::warn!("failed to open sound file {path:?}: {e}");
                                continue;
                            }
                        };
                        let decoder = match rodio::Decoder::new(BufReader::new(file)) {
                            Ok(d) => d,
                            Err(e) => {
                                log::warn!("failed to decode sound file {path:?}: {e}");
                                continue;
                            }
                        };
                        match rodio::Sink::try_new(handle) {
                            Ok(sink) => {
                                sink.set_volume(volume.clamp(0.0, 1.0));
                                sink.append(decoder);
                                active_sinks.push(sink);
                            }
                            Err(e) => log::warn!("failed to create audio sink: {e}"),
                        }
                    }
                    AudioCommand::StopAll => {
                        for sink in active_sinks.drain(..) {
                            sink.stop();
                        }
                    }
                }
            }
        });

        Self { tx }
    }

    /// Queues playback and returns immediately; never blocks the caller.
    pub fn play(&self, path: &Path, volume: f32) {
        let _ = self.tx.send(AudioCommand::Play {
            path: path.to_path_buf(),
            volume,
        });
    }

    /// Immediately silences anything currently playing — the Click Studio
    /// "master disable" switch.
    pub fn stop_all(&self) {
        let _ = self.tx.send(AudioCommand::StopAll);
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidatedAudio {
    pub duration_ms: Option<u32>,
}

/// Validates an uploaded sound file before it's copied into app-controlled
/// storage: extension allow-list, size cap, and a real decode attempt so a
/// renamed non-audio file or a corrupted upload is rejected up front rather
/// than failing silently the first time someone clicks.
pub fn validate_audio_file(path: &Path) -> AppResult<ValidatedAudio> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| AppError::InvalidInput("file has no extension".into()))?;

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "unsupported file type \".{ext}\" — only WAV, MP3, and OGG are supported"
        )));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| AppError::InvalidInput(format!("cannot read file: {e}")))?;
    if metadata.len() == 0 {
        return Err(AppError::InvalidInput("file is empty".into()));
    }
    if metadata.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::InvalidInput(format!(
            "file is too large ({:.1} MB, limit is {:.0} MB)",
            metadata.len() as f64 / 1_048_576.0,
            MAX_UPLOAD_BYTES as f64 / 1_048_576.0
        )));
    }

    // WAV header gives us an exact duration; other formats are duration-checked
    // best-effort via the decoder (not always available without a full scan).
    let duration_ms = if ext == "wav" {
        let reader = hound::WavReader::open(path)
            .map_err(|e| AppError::InvalidInput(format!("not a valid WAV file: {e}")))?;
        let spec = reader.spec();
        let frames = reader.duration();
        let ms = (frames as u64 * 1000) / spec.sample_rate.max(1) as u64;
        Some(ms as u32)
    } else {
        let file = std::fs::File::open(path)
            .map_err(|e| AppError::InvalidInput(format!("cannot open file: {e}")))?;
        let decoder = rodio::Decoder::new(BufReader::new(file))
            .map_err(|e| AppError::InvalidInput(format!("could not decode audio: {e}")))?;
        decoder.total_duration().map(|d: Duration| d.as_millis() as u32)
    };

    if let Some(ms) = duration_ms {
        if ms > MAX_DURATION_MS {
            return Err(AppError::InvalidInput(format!(
                "sound is {:.1}s long, limit is {:.0}s",
                ms as f64 / 1000.0,
                MAX_DURATION_MS as f64 / 1000.0
            )));
        }
    }

    Ok(ValidatedAudio { duration_ms })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_minimal_wav(path: &Path, duration_secs: f32) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let n = (44100.0 * duration_secs) as u32;
        for i in 0..n {
            let v = ((i as f32 * 0.1).sin() * 1000.0) as i16;
            writer.write_sample(v).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn accepts_short_valid_wav() {
        let dir = std::env::temp_dir().join(format!("glooblet-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("click.wav");
        write_minimal_wav(&path, 0.05);
        let result = validate_audio_file(&path).unwrap();
        assert!(result.duration_ms.unwrap() < 200);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_oversized_duration() {
        let dir = std::env::temp_dir().join(format!("glooblet-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("long.wav");
        write_minimal_wav(&path, 9.0);
        let err = validate_audio_file(&path).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_unsupported_extension() {
        let dir = std::env::temp_dir().join(format!("glooblet-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("click.exe");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"not audio").unwrap();
        let err = validate_audio_file(&path).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_corrupted_wav() {
        let dir = std::env::temp_dir().join(format!("glooblet-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("corrupt.wav");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"RIFF not really a wav file").unwrap();
        let err = validate_audio_file(&path).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        std::fs::remove_dir_all(&dir).ok();
    }
}
