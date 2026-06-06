use crate::error::AudioError;

pub mod audio;
pub mod device;
pub mod error;
pub mod stream;

pub type PvcAudioId = u32;

pub type Result<T> = std::result::Result<T, AudioError>;
