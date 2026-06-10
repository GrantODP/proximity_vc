use crossbeam::queue::SegQueue;

use crate::stream::AudioSlice;
use crate::traits::{AudioBuffer, AudioSink, InputReader, InputWriter};
use crate::util::{receive_audio, write_audio};
use crate::{Result, stream};

pub struct AudioUnboundQueue {
    pub channels: u32,
    data: SegQueue<f32>,
}

impl AudioUnboundQueue {
    pub fn new(channels: u32) -> Self {
        Self {
            channels,
            data: SegQueue::new(),
        }
    }

    pub fn clear(&mut self) {
        while self.data.pop().is_some() {}
    }
}

impl AudioBuffer for AudioUnboundQueue {
    fn channels(&self) -> u32 {
        self.channels
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl InputWriter for AudioUnboundQueue {
    fn write(&self, sample: f32) -> Result<()> {
        self.data.push(sample);
        Ok(())
    }

    fn write_slice(&self, slice: &[f32]) -> usize {
        for sample in slice.iter() {
            self.data.push(*sample);
        }
        slice.len()
    }
}
impl InputReader for AudioUnboundQueue {
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

impl AudioSink for AudioUnboundQueue {
    fn on_read(&self, slice: AudioSlice<'_>) {
        receive_audio(self, slice);
    }

    fn on_write(&self, slice: stream::AudioSliceMut<'_>) {
        write_audio(self, slice);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_unbound_buffer() {
        let mut buffer = AudioUnboundQueue::new(2);
        assert_eq!(buffer.channels, 2);
        assert!(buffer.read().is_none());
        buffer.write(0.5);
        assert!(buffer.read().is_some());
        buffer.write(0.5);
        assert!(buffer.read().is_some());
        assert!(buffer.read().is_none());

        let slice = [0.5; 20];
        buffer.write_slice(&slice);
        assert!(buffer.read().is_some());

        buffer.clear();
        assert!(buffer.read().is_none());
    }
}
