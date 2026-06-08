use std::sync::Arc;

use crate::{
    Result,
    device::{self, AudioDevice, Input},
    error::AudioError,
    stream::{self, AudioSink, AudioSlice},
};
use crossbeam::queue::ArrayQueue;

const AUDIOBUFFER_SIZE: usize = 48000 * 2;

///Ring buffer for f32 audio samples
///This buffer is used to store audio samples for processing.
///User can write audio samples directly to buffer using [`InputWriter`] trait.
///However the primary use to insert audio samples from an input stream in [`InputStream`]
///[`AudioRingBuffer`] is uses a MPMPC queue as its underlying container.
#[derive(Debug)]
pub struct AudioRingBuffer {
    ///Number of channels in the buffer
    pub channels: u32,
    ///The underlying ring buffer
    pub data: ArrayQueue<f32>,
}

impl AudioRingBuffer {
    ///Creates a new AudioRingBuffer with the specified number of channels and buffer size.
    /// # Arguments
    /// * `channels` - The number of audio channels
    /// * `buffer_size` - The size of the buffer in samples
    /// # Returns
    /// * `Self` - The new AudioRingBuffer
    /// # Examples
    /// ```
    /// let writer = AudioRingBuffer::new(1, AUDIOBUFFER_SIZE);
    /// ```
    pub fn new(channels: u32, buffer_size: usize) -> Self {
        let data = ArrayQueue::new(buffer_size);
        Self {
            channels,
            data: data.into(),
        }
    }

    ///Clears the buffer, removing all audio samples.
    ///Note if any operations are writing to the buffer, this may block causing and infinite loop.
    pub fn clear(&mut self) {
        while self.data.pop().is_some() {}
    }
}

///Simple trait to mark types as input readers, providing audio samples.
pub trait InputReader {
    ///Reads a single audio sample from the buffer.
    /// Returns `0.0` if no sample is available.
    fn read(&self) -> Option<f32>;

    ///Reads a slice of audio samples from the buffer.
    /// Returns the number of samples read.
    fn read_slice(&self, slice: &mut [f32]) -> usize;
}
///Simple trait to mark types as input writer, to push audio samples into the buffer.
pub trait InputWriter {
    ///Writes a single audio sample to the buffer.
    /// # Arguments
    /// * `value` - The f32 audio sample
    /// # Returns
    /// * `Ok(())` - The sample was written successfully.
    /// * `Err(AudioError)` - The sample could not be written.
    /// # Examples
    /// ```
    /// let writer = AudioRingBuffer::new(1, 1024);
    /// writer.write(1.0).unwrap();
    /// ```
    fn write(&self, sample: f32) -> Result<()>;

    ///Writes a slice of audio samples to the buffer.
    /// # Arguments
    /// * `slice` - The slice of f32 audio samples to write.
    /// # Returns
    /// * `Ok(())` - The samples were written successfully.
    /// * `Err(AudioError)` - The samples could not be written.
    /// # Examples
    /// ```
    /// let writer = AudioRingBuffer::new(1, 1024);
    /// writer.write_slice(&[1.0, 2.0, 3.0]).unwrap();
    /// ```
    fn write_slice(&self, slice: &[f32]) -> usize;
}

// utility function convert other audio formats to f32
// TODO: Find better method to convert audio formats to f32
fn receive_audio<T>(buffer: &T, slice: AudioSlice<'_>)
where
    T: InputWriter,
{
    match slice {
        AudioSlice::I8(items) => {
            for &s in items {
                let f = (s as f32) / 128.0;
                let _ = buffer.write(f);
            }
        }

        AudioSlice::I16(items) => {
            for &s in items {
                let f = (s as f32) / 32768.0;
                let _ = buffer.write(f);
            }
        }

        AudioSlice::I32(items) => {
            for &s in items {
                let f = (s as f32) / 2_147_483_648.0;
                let _ = buffer.write(f);
            }
        }

        AudioSlice::F32(items) => {
            buffer.write_slice(items);
        }
    }
}
// Assign ringbuffer as an observer for an input stream.
impl AudioSink for AudioRingBuffer {
    fn on_read(&self, slice: AudioSlice<'_>) {
        receive_audio(self, slice);
    }

    fn on_write(&self, slice: stream::AudioSliceMut<'_>) {}
}

impl InputReader for AudioRingBuffer {
    fn read(&self) -> Option<f32> {
        self.data.pop()
    }

    fn read_slice(&self, slice: &mut [f32]) -> usize {
        let mut count = 0;
        for sample in slice.iter_mut() {
            if let Some(value) = self.data.pop() {
                *sample = value;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}
impl InputWriter for AudioRingBuffer {
    ///Writes a single audio sample to the buffer.
    ///This will overwrite the oldest sample if the buffer is full.
    /// # Arguments
    /// * `value` - The f32 audio sample
    /// # Returns
    /// * `Ok(())` - The sample was written successfully.
    /// * `Err(AudioError)` - The sample could not be written.
    /// # Examples
    /// ```
    /// let writer = AudioRingBuffer::new(1, AUDIOBUFFER_SIZE);
    /// writer.write(1.0).unwrap();
    /// ```
    fn write(&self, value: f32) -> Result<()> {
        self.data.force_push(value);
        Ok(())
    }

    ///Writes a slice of audio samples to the buffer.
    ///This will overwrite the oldest samples if the buffer is full.
    /// # Arguments
    /// * `slice` - The slice of f32 audio samples
    /// # Returns
    /// * `usize` - The number of samples written.
    /// # Examples
    /// ```
    /// let writer = AudioRingBuffer::new(1, AUDIOBUFFER_SIZE);
    /// writer.write_slice(&[1.0, 2.0, 3.0]).unwrap();
    /// ```
    fn write_slice(&self, slice: &[f32]) -> usize {
        for sample in slice.iter() {
            self.data.force_push(*sample);
        }
        // will always return the full slice length because of overwrite.
        slice.len()
    }
}

#[derive(Debug)]
pub struct AudioFixedQueue {
    channels: u32,
    data: ArrayQueue<f32>,
}

impl AudioFixedQueue {
    pub fn new(channels: u32, capacity: usize) -> Self {
        Self {
            channels,
            data: ArrayQueue::new(capacity),
        }
    }
}

impl InputWriter for AudioFixedQueue {
    fn write(&self, sample: f32) -> Result<()> {
        let result = self.data.push(sample);
        result.map_err(|_| AudioError::AudioBufferFull)
    }

    fn write_slice(&self, slice: &[f32]) -> usize {
        let mut count = 0;
        for sample in slice.iter() {
            let result = self.data.push(*sample);
            if result.is_ok() {
                count += 1;
            }
        }
        count
    }
}
impl InputReader for AudioFixedQueue {
    fn read(&self) -> Option<f32> {
        self.data.pop()
    }

    fn read_slice(&self, slice: &mut [f32]) -> usize {
        let mut count = 0;
        for sample in slice.iter_mut() {
            if let Some(s) = self.data.pop() {
                *sample = s;
                count += 1;
            }
        }
        count
    }
}
/// Represents the kind of buffer used by [`AudioBufferBuilder`] to build an [`AudioBuffer`].
#[derive(Debug, Clone, Copy, Default)]
pub enum BufferKind {
    #[default]
    Ring, // AudioRingBuffer
    Fixed, // AudioFixedQueue
    Grow,  //TODO
}

///Container used by builders to return various AudioBuffer types.
/// Used to simplify the creation of buffers that require eithe
pub enum AudioBuffers {
    Ring(AudioRingBuffer),
    Fixed(AudioFixedQueue),
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
            BufferKind::Grow => todo!(),
        }
    }
}
