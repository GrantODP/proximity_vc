use crate::{
    buffers::*,
    device::{self, AudioDevice, Input},
};

/// Represents the kind of buffer used by [`AudioBufferBuilder`] to build an [`AudioBuffer`].
#[derive(Debug, Clone, Copy, Default)]
pub enum BufferKind {
    #[default]
    Ring, // AudioRingBuffer
    Fixed,   // AudioFixedQueue
    Unbound, // AudioUnboundQueue
}

///Container used by builders to return various AudioBuffer types.
/// Used to simplify the creation of buffers that require eithe
pub enum AudioBuffers {
    Ring(AudioRingBuffer),
    Fixed(AudioFixedQueue),
    Unbound(AudioUnboundQueue),
}

///Builder for creating [`AudioBuffers`] of various kinds.
#[derive(Debug, Clone, Copy)]
pub struct AudioBufferBuilder;

impl AudioBufferBuilder {
    ///Returns a [`BufferBuilder`] for input buffers.
    pub fn input() -> BufferBuilder<device::Input> {
        BufferBuilder::<device::Input>::new(BufferKind::Ring)
    }
    ///Returns a [`BufferBuilder`] for output buffers.
    pub fn output() -> BufferBuilder<device::Output> {
        BufferBuilder::<device::Output>::new(BufferKind::Ring)
    }
}

const AUDIOBUFFER_SIZE: usize = 48000 * 2;
#[derive(Debug, Default)]
pub struct BufferBuilder<T> {
    kind: BufferKind,
    buffer_size: usize,
    _state: std::marker::PhantomData<T>,
}

impl<T> BufferBuilder<T> {
    ///Returns a [`BufferBuilder`] with the specified [`BufferKind`].
    pub fn new(kind: BufferKind) -> Self {
        Self {
            kind,
            buffer_size: AUDIOBUFFER_SIZE,
            _state: std::marker::PhantomData,
        }
    }
    ///Returns a [`BufferBuilder`] with the specified buffer size.
    pub fn set_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    ///Returns a [`BufferBuilder`] with the specified [`BufferKind`].
    pub fn set_kind(mut self, kind: BufferKind) -> Self {
        self.kind = kind;
        self
    }
}
///Build for input audio buffers.
impl BufferBuilder<Input> {
    /// Builds an [`AudioBuffers`] designed for input audio.
    pub fn build(self, device: &AudioDevice<Input>) -> AudioBuffers {
        match self.kind {
            BufferKind::Ring => {
                let buff = AudioRingBuffer::new(device.config.channels().into(), self.buffer_size);
                let buff_type = AudioBuffers::Ring(buff);
                buff_type
            }
            BufferKind::Fixed => {
                let buff = AudioFixedQueue::new(device.config.channels().into(), self.buffer_size);
                let buff_type = AudioBuffers::Fixed(buff);
                buff_type
            }
            BufferKind::Unbound => {
                let buff = AudioUnboundQueue::new(device.config.channels().into());
                let buff_type = AudioBuffers::Unbound(buff);
                buff_type
            }
        }
    }
}
