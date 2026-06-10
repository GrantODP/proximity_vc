// Assumes the input f32 is in the range [-1.0, 1.0].
use crate::{
    stream::{AudioSlice, AudioSliceMut},
    traits::{AudioBuffer, InputReader, InputWriter},
};
use cpal::{FromSample, Sample};

// utility function convert other audio formats to f32
// TODO: Find better method to convert audio formats to f32

const CHUNK_SIZE: usize = 256;

pub fn receive_audio<T>(buffer: &T, slice: AudioSlice<'_>)
where
    T: InputWriter + AudioBuffer,
{
    // 1. REMOVED println!() - It triggers thread-blocking OS I/O locks.
    println!("Received slice");
    match slice {
        AudioSlice::F32(items) => {
            buffer.write_slice(items);
        }
        _ => {
            // 2. Vectorized Block processing: Use an intermediate stack array
            let mut scratch = [0.0f32; CHUNK_SIZE];

            // Map our enum variant items directly down to a universal macro
            // handler to allow compiler Auto-Vectorization (SIMD)
            macro_rules! convert_and_write {
                ($items:expr) => {
                    for chunk in $items.chunks(CHUNK_SIZE) {
                        for (i, &s) in chunk.iter().enumerate() {
                            scratch[i] = s.to_sample::<f32>();
                        }
                        // Batch write the transformed slice all at once
                        buffer.write_slice(&scratch[..chunk.len()]);
                    }
                };
            }

            match slice {
                AudioSlice::I8(items) => convert_and_write!(items),
                AudioSlice::I16(items) => convert_and_write!(items),
                AudioSlice::I32(items) => convert_and_write!(items),
                AudioSlice::I24(items) => convert_and_write!(items),
                AudioSlice::U24(items) => convert_and_write!(items),
                AudioSlice::I64(items) => convert_and_write!(items),
                AudioSlice::U64(items) => convert_and_write!(items),
                AudioSlice::DsdU8(items) => convert_and_write!(items),
                AudioSlice::DsdU16(items) => convert_and_write!(items),
                AudioSlice::DsdU32(items) => convert_and_write!(items),
                AudioSlice::F32(_) => unreachable!(),
            }
        }
    }
}

pub fn write_audio<T>(buffer: &T, slice: AudioSliceMut<'_>)
where
    T: InputReader + AudioBuffer,
{
    println!("Writing audio");
    match slice {
        AudioSliceMut::F32(items) => {
            // Handled efficiently natively via slice copies
            buffer.read_slice(items);
        }
        _ => {
            // let mut scratch = [0.0f32; CHUNK_SIZE];
            macro_rules! read_and_convert {
                ($items:expr, $t:ty) => {
                    // Iterate sequentially instead of slicing chunks by variable channels
                    for chunk in $items {
                        let sample = buffer.read().unwrap_or(0.0);
                        *chunk = <$t>::from_sample(sample);
                    }
                };
            }

            match slice {
                AudioSliceMut::I8(items) => read_and_convert!(items, i8),
                AudioSliceMut::I16(items) => read_and_convert!(items, i16),
                AudioSliceMut::I32(items) => read_and_convert!(items, i32),
                AudioSliceMut::I24(items) => read_and_convert!(items, i32),
                AudioSliceMut::U24(items) => read_and_convert!(items, u32),
                AudioSliceMut::I64(items) => read_and_convert!(items, i64),
                AudioSliceMut::U64(items) => read_and_convert!(items, u64),
                AudioSliceMut::DsdU8(items) => read_and_convert!(items, u8),
                AudioSliceMut::DsdU16(items) => read_and_convert!(items, u16),
                AudioSliceMut::DsdU32(items) => read_and_convert!(items, u32),
                AudioSliceMut::F32(_) => unreachable!(),
            }
        }
    }
}
