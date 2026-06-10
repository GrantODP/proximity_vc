use crossbeam::queue::ArrayQueue;

use crate::{
    Result,
    error::AudioError,
    stream::{AudioSlice, AudioSliceMut},
    traits::{AudioBuffer, AudioSink, InputReader, InputWriter},
    util::receive_audio,
};

#[derive(Debug)]
pub struct AudioFixedQueue {
    pub channels: u32,
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
impl AudioBuffer for AudioFixedQueue {
    fn channels(&self) -> u32 {
        self.channels
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl InputWriter for AudioFixedQueue {
    fn write(&self, sample: f32) -> Result<()> {
        let result = self.data.push(sample);
        result.map_err(|_| AudioError::BufferFull)
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
impl AudioSink for AudioFixedQueue {
    fn on_read(&self, slice: AudioSlice<'_>) {
        receive_audio(self, slice);
    }

    fn on_write(&self, slice: AudioSliceMut<'_>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_fixed_queue() {
        let queue = AudioFixedQueue::new(2, 20);
        assert_eq!(queue.channels, 2);
        assert_eq!(queue.data.capacity(), 20);
        assert!(queue.data.is_empty());

        let samples = [0.5; 20];
        let r = queue.write(samples[0]);
        assert!(r.is_ok());
        assert!(!queue.data.is_empty());
        assert_eq!(queue.data.len(), 1);

        let r = queue.write_slice(&samples[1..20]);
        assert!(r == 19);

        let r = queue.write(0.0);
        assert!(r.is_err());
    }
}
