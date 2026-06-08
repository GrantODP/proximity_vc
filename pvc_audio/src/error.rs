use cpal::{
    BuildStreamError, DeviceIdError, DeviceNameError, DevicesError, PauseStreamError,
    PlayStreamError, StreamError, SupportedStreamConfigsError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("DeviceError")]
    DeviceError(#[from] DevicesError),

    #[error("DeviceError")]
    DeviceNameError(#[from] DeviceNameError),

    #[error("DeviceError")]
    DeviceIdError(#[from] DeviceIdError),

    #[error("DeviceError")]
    InputConfigs(#[from] SupportedStreamConfigsError),

    #[error("DeviceError")]
    BuildStreamError(#[from] BuildStreamError),

    #[error("DeviceError")]
    StreamError(#[from] StreamError),

    #[error("No audio devices found")]
    NoAudioDevice,

    #[error("UnsupportedSampleFormat {0}")]
    UnsupportedSampleFormat(String),

    #[error("No info for audio device")]
    NoInfoForAudioDevice,

    #[error("PlayStreamError")]
    PlayStreamError(#[from] PlayStreamError),

    #[error("PauseStreamError")]
    PauseStreamError(#[from] PauseStreamError),

    #[error("AudioBufferFull")]
    AudioBufferFull,
}
