use thiserror::Error;
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("")]
    BackendError(#[from] cpal::Error),

    #[error("Audio Buffer is Full")]
    BufferFull,

    #[error("No supported configs found for device {0}")]
    NoInfoForAudioDevice(String),

    #[error("Unsupported sample format {0}")]
    UnsupportedSampleFormat(String),
}
