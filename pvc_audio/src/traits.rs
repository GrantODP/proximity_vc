use crate::Result;
use crate::stream::{AudioSlice, AudioSliceMut};

/// Types that need to receive memory points that can read data from the input stream and write data to the output stream.
pub trait AudioSink: Send + Sync {
    /// Called when audio data is read from the input stream.
    fn on_read(&self, slice: AudioSlice<'_>);

    /// Called when data can be written to the output stream.
    /// Will receive a mutable slice of memory points to write audio data to the output stream.
    fn on_write(&self, slice: AudioSliceMut<'_>);
}

pub trait AudioBuffer {
    fn channels(&self) -> u32;

    fn len(&self) -> usize;
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
