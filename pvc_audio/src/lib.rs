use crate::error::AudioError;

pub mod buffers;
pub mod builder;
pub mod device;
pub mod error;
pub mod stream;
pub mod traits;
pub mod util;

pub type PvcAudioId = u32;

pub type Result<T> = std::result::Result<T, AudioError>;
