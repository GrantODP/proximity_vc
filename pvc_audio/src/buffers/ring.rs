use crate::stream::AudioSlice;
use crate::traits::{AudioBuffer, AudioSink, InputReader, InputWriter};
use crate::util::{receive_audio, write_audio};
use crate::{
    Result,
    stream::{self},
};
use crossbeam::queue::ArrayQueue;

const AUDIOBUFFER_SIZE: usize = 48000 * 2;

///Ring buffer for f32 audio samples.
///This buffer is used to store audio samples for processing.
///User can write audio samples directly to buffer using [`InputWriter`] trait.
///However the primary use is to insert audio samples from an input stream in [`InputStream`]
///[`AudioRingBuffer`] is uses a MPMC queue as its underlying container.
#[derive(Debug)]
pub struct AudioRingBuffer {
    ///Number of channels in the buffer
    pub channels: u32,
    ///The underlying ring buffer
    data: ArrayQueue<f32>,
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

// Assign ringbuffer as an observer for an input stream.
impl AudioSink for AudioRingBuffer {
    fn on_read(&self, slice: AudioSlice<'_>) {
        receive_audio(self, slice);
    }

    fn on_write(&self, slice: stream::AudioSliceMut<'_>) {
        write_audio(self, slice);
    }
}

impl AudioBuffer for AudioRingBuffer {
    fn channels(&self) -> u32 {
        self.channels
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl InputReader for AudioRingBuffer {
    fn read(&self) -> Option<f32> {
        self.data.pop()
    }

    fn read_slice(&self, slice: &mut [f32]) -> usize {
        let mut count = 0;
        let target_len = slice.len();

        // 1. Optimize chunk processing by unrolling loops manually
        // or processing in blocks to reduce loop condition checks
        while count < target_len {
            let remaining = target_len - count;

            // Process in small batches of 4 to maximize CPU pipeline efficiency
            if remaining >= 4 {
                if let (Some(v0), Some(v1), Some(v2), Some(v3)) = (
                    self.data.pop(),
                    self.data.pop(),
                    self.data.pop(),
                    self.data.pop(),
                ) {
                    slice[count] = v0;
                    slice[count + 1] = v1;
                    slice[count + 2] = v2;
                    slice[count + 3] = v3;
                    count += 4;
                    continue;
                }
            }

            // Fallback for single samples or remaining items
            if let Some(value) = self.data.pop() {
                slice[count] = value;
                count += 1;
            } else {
                break; // Queue is totally empty
            }
        }
        println!("OUT {:?}", self.data.len());
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
        println!(" SAMPLE{:?}", value);
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
            println!(" SAMPLE{:?}", sample);
            self.data.force_push(*sample);
        }
        println!("IN {:?}", self.data.len());
        // will always return the full slice length because of overwrite.
        slice.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_ring_buffer() {
        let mut buffer = AudioRingBuffer::new(2, 20);
        assert_eq!(buffer.channels, 2);
        // assert_eq!(buffer.data.capacity(), 20);
        // assert!(buffer.data.is_empty());

        let samples = [0.5; 20];
        buffer.write_slice(&samples);
        while let Some(v) = buffer.read() {
            assert_eq!(v, 0.5)
        }
        // assert!(!buffer.data.is_empty());
        // assert_eq!(buffer.data.len(), 20);

        let first = buffer.read();
        if let Some(first) = first {
            assert_eq!(first, 0.5);
        } else {
            panic!("Expected first sample to be Some(0.5)");
        }
        while let Some(v) = buffer.read() {
            assert_eq!(v, 0.5)
        }
        let samples = [0.5; 20];
        buffer.write_slice(&samples);

        let mut second_pops = [0.0; 5];
        buffer.read_slice(&mut second_pops);
        assert_eq!(second_pops, [0.5; 5]);

        buffer.clear();
        assert!(buffer.read().is_none());

        //overflow should overwrite oldest samples
        buffer.write_slice(&samples);
        let r = buffer.write(0.1);
        assert!(r.is_ok());
        let mut old_samples = [0.0; 19];
        let count = buffer.read_slice(&mut old_samples);
        assert_eq!(count, 19);
        let last = buffer.read();
        assert!(last.is_some());
        assert_eq!(last.unwrap(), 0.1);
    }
}
