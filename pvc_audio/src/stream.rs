use std::sync::{Arc, RwLock};

use cpal::{
    Device, FromSample, Sample, SampleFormat, Stream, StreamConfig, SupportedStreamConfig,
    traits::{DeviceTrait, StreamTrait},
};

use crate::{Result, error::AudioError, traits::AudioSink};

/// A slice of audio data in the format specified by the device/inputstream
#[derive(Debug, Clone)]
pub enum AudioSlice<'a> {
    I8(&'a [i8]),
    I16(&'a [i16]),
    I32(&'a [i32]),
    F32(&'a [f32]),
    I24(&'a [i32]),
    U24(&'a [u32]),
    I64(&'a [i64]),
    U64(&'a [u64]),
    DsdU8(&'a [u8]),
    DsdU16(&'a [u16]),
    DsdU32(&'a [u32]),
}

/// A mutable slice of audio points used to write audio data to the output stream.
#[derive(Debug)]
pub enum AudioSliceMut<'a> {
    I8(&'a mut [i8]),
    I16(&'a mut [i16]),
    I32(&'a mut [i32]),
    F32(&'a mut [f32]),
    I24(&'a mut [i32]),
    U24(&'a mut [u32]),
    I64(&'a mut [i64]),
    U64(&'a mut [u64]),
    DsdU8(&'a mut [u8]),
    DsdU16(&'a mut [u16]),
    DsdU32(&'a mut [u32]),
}

struct AudioDispatcher {
    pub sinks: RwLock<Vec<Arc<dyn AudioSink>>>,
}

pub struct InputStreamHandle {
    stream: Stream,
    dispatcher: Arc<AudioDispatcher>, //For all cases there should normaly be exactly of 2 references of the Dispatcher 1 owned by the handle and the other is owned by the stream callback
}

pub struct OutputStreamHandle {
    stream: Stream,
}

impl AudioDispatcher {
    fn new() -> Self {
        Self {
            sinks: RwLock::new(vec![]),
        }
    }
    fn on_input(&self, slice: AudioSlice<'_>) {
        for sink in self.sinks.read().unwrap().iter() {
            sink.on_read(slice.clone());
        }
    }

    fn push(&self, sink: Arc<dyn AudioSink>) {
        let mut w_guard = self.sinks.write().unwrap();
        w_guard.push(sink);
    }
}

impl InputStreamHandle {
    /// Pushes a sink to the dispatcher, so that it will receive audio data from the stream.
    ///
    /// # Arguments
    ///
    /// * `sink` - The sink to push.
    ///
    /// # Examples
    ///
    /// ```
    /// // Buffer implements [`AudioSink`]
    /// let sink: Arc<dyn AudioSink> = Arc::new(Buffer::new());
    /// let handle = build_input_handle(&device, config, |_| {}).unwrap();
    /// handle.push_sink(sink.clone());
    /// ```
    pub fn push_sink(&self, sink: Arc<dyn AudioSink>) {
        self.dispatcher.push(sink);
    }

    /// The stream will open and start running
    pub fn play(&self) -> Result<()> {
        Ok(self.stream.play()?)
    }

    pub fn pause(&self) -> Result<()> {
        Ok(self.stream.pause()?)
    }

    pub fn stream(&self) -> &Stream {
        &self.stream
    }
}
impl OutputStreamHandle {
    /// The stream will open and start running
    pub fn play(&self) -> Result<()> {
        Ok(self.stream.play()?)
    }

    pub fn pause(&self) -> Result<()> {
        Ok(self.stream.pause()?)
    }

    pub fn stream(&self) -> &Stream {
        &self.stream
    }
}

pub fn build_input_handle<E>(
    device: &Device,
    config: SupportedStreamConfig,
    on_err: E,
) -> Result<InputStreamHandle>
where
    E: FnMut(cpal::Error) + Send + 'static,
{
    let dispatcher = Arc::new(AudioDispatcher::new());
    let dsp2 = dispatcher.clone();
    let stream = build_input_stream(
        device,
        config,
        move |slice| {
            dispatcher.on_input(slice);
        },
        on_err,
    )?;

    Ok(InputStreamHandle {
        stream: stream,
        dispatcher: dsp2,
    })
}
pub fn build_output_handle<E>(
    device: &Device,
    config: SupportedStreamConfig,
    buffer: Arc<dyn AudioSink>,
    on_err: E,
) -> Result<OutputStreamHandle>
where
    E: FnMut(cpal::Error) + Send + 'static,
{
    let stream = build_output_stream(
        device,
        config,
        move |slice| {
            buffer.on_write(slice);
        },
        on_err,
    )?;
    Ok(OutputStreamHandle { stream: stream })
}
pub fn build_input_stream<D, E>(
    device: &Device,
    config: SupportedStreamConfig,
    mut on_data: D,
    on_err: E,
) -> Result<Stream>
where
    D: FnMut(AudioSlice<'_>) + Send + 'static,
    E: FnMut(cpal::Error) + Send + 'static,
{
    let format = config.sample_format();
    let stream_config: StreamConfig = config.into();

    let stream = match format {
        SampleFormat::I8 => device.build_input_stream(
            stream_config,
            move |data: &[i8], _| on_data(AudioSlice::I8(data)),
            on_err,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            stream_config,
            move |data: &[i16], _| on_data(AudioSlice::I16(data)),
            on_err,
            None,
        )?,
        SampleFormat::I24 => device.build_input_stream(
            stream_config,
            move |data: &[i32], _| on_data(AudioSlice::I24(data)),
            on_err,
            None,
        )?,
        SampleFormat::U24 => device.build_input_stream(
            stream_config,
            move |data: &[u32], _| on_data(AudioSlice::U24(data)),
            on_err,
            None,
        )?,
        SampleFormat::I32 => device.build_input_stream(
            stream_config,
            move |data: &[i32], _| on_data(AudioSlice::I32(data)),
            on_err,
            None,
        )?,
        SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| on_data(AudioSlice::F32(data)),
            on_err,
            None,
        )?,
        SampleFormat::I64 => device.build_input_stream(
            stream_config,
            move |data: &[i64], _| on_data(AudioSlice::I64(data)),
            on_err,
            None,
        )?,
        SampleFormat::U64 => device.build_input_stream(
            stream_config,
            move |data: &[u64], _| on_data(AudioSlice::U64(data)),
            on_err,
            None,
        )?,
        SampleFormat::DsdU8 => device.build_input_stream(
            stream_config,
            move |data: &[u8], _| on_data(AudioSlice::DsdU8(data)),
            on_err,
            None,
        )?,
        SampleFormat::DsdU16 => device.build_input_stream(
            stream_config,
            move |data: &[u16], _| on_data(AudioSlice::DsdU16(data)),
            on_err,
            None,
        )?,
        SampleFormat::DsdU32 => device.build_input_stream(
            stream_config,
            move |data: &[u32], _| on_data(AudioSlice::DsdU32(data)),
            on_err,
            None,
        )?,
        fmt => {
            return Err(AudioError::UnsupportedSampleFormat(format!(
                "Sample format {:?} is unsupported",
                fmt
            )));
        }
    };

    Ok(stream)
}

pub fn build_output_stream<D, E>(
    device: &Device,
    config: SupportedStreamConfig,
    mut on_data: D,
    on_err: E,
) -> Result<Stream>
where
    D: FnMut(AudioSliceMut<'_>) + Send + 'static,
    E: FnMut(cpal::Error) + Send + 'static,
{
    let format = config.sample_format();
    let stream_config: StreamConfig = config.into();

    let stream = match format {
        SampleFormat::I8 => device.build_output_stream(
            stream_config,
            move |data: &mut [i8], _| on_data(AudioSliceMut::I8(data)),
            on_err,
            None,
        )?,
        SampleFormat::I16 => device.build_output_stream(
            stream_config,
            move |data: &mut [i16], _| on_data(AudioSliceMut::I16(data)),
            on_err,
            None,
        )?,
        SampleFormat::I24 => device.build_output_stream(
            stream_config,
            move |data: &mut [i32], _| on_data(AudioSliceMut::I24(data)),
            on_err,
            None,
        )?,
        SampleFormat::U24 => device.build_output_stream(
            stream_config,
            move |data: &mut [u32], _| on_data(AudioSliceMut::U24(data)),
            on_err,
            None,
        )?,
        SampleFormat::I32 => device.build_output_stream(
            stream_config,
            move |data: &mut [i32], _| on_data(AudioSliceMut::I32(data)),
            on_err,
            None,
        )?,
        SampleFormat::F32 => device.build_output_stream(
            stream_config,
            move |data: &mut [f32], _| on_data(AudioSliceMut::F32(data)),
            on_err,
            None,
        )?,
        SampleFormat::I64 => device.build_output_stream(
            stream_config,
            move |data: &mut [i64], _| on_data(AudioSliceMut::I64(data)),
            on_err,
            None,
        )?,
        SampleFormat::U64 => device.build_output_stream(
            stream_config,
            move |data: &mut [u64], _| on_data(AudioSliceMut::U64(data)),
            on_err,
            None,
        )?,
        SampleFormat::DsdU8 => device.build_output_stream(
            stream_config,
            move |data: &mut [u8], _| on_data(AudioSliceMut::DsdU8(data)),
            on_err,
            None,
        )?,
        SampleFormat::DsdU16 => device.build_output_stream(
            stream_config,
            move |data: &mut [u16], _| on_data(AudioSliceMut::DsdU16(data)),
            on_err,
            None,
        )?,
        SampleFormat::DsdU32 => device.build_output_stream(
            stream_config,
            move |data: &mut [u32], _| on_data(AudioSliceMut::DsdU32(data)),
            on_err,
            None,
        )?,
        fmt => {
            return Err(AudioError::UnsupportedSampleFormat(format!(
                "Sample format {:?} is unsupported",
                fmt
            )));
        }
    };

    Ok(stream)
}
#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use cpal::traits::{HostTrait, StreamTrait};
    use ringbuf::{
        HeapRb,
        traits::{Consumer, Producer, Split},
    };

    use super::*;

    //Physcial mic test not to be used in unit tests
    #[test]
    #[ignore]
    fn test_input_audio() {
        println!("TESTING");
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();

        let stream = build_input_stream(
            &device,
            config.clone(),
            |audio_slice| match audio_slice {
                AudioSlice::F32(slice) => {
                    println!("Got {} f32 samples {:?}", slice.len(), &slice[0..5])
                }
                AudioSlice::I16(slice) => {
                    println!("Got {} i16 samples {:?}", slice.len(), &slice[0..5])
                }
                _ => {
                    println!("Got ???");
                }
            },
            |err| eprintln!("Stream error: {err}"),
        )
        .unwrap();

        stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(stream);
    }

    //Physcial speaker test not to be used in unit tests, this will play a beep as the example from cpal/examples/beep.rs
    #[test]
    fn test_output_audio() {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate() as f32;
        let channels: usize = config.channels().into();
        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };
        let stream = build_output_stream(
            &device,
            config.clone(),
            move |audio_slice| match audio_slice {
                AudioSliceMut::F32(slice) => {
                    println!("Playing beep");
                    for frame in slice.chunks_mut(channels) {
                        let value = next_value();
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                }
                _ => {
                    println!("Got ???");
                }
            },
            |err| eprintln!("Stream error: {err}"),
        )
        .unwrap();

        stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(stream);
    }

    //Input to output stream, physical test for SPSC buffer
    #[test]
    fn test_audio() {
        println!("TESTING");
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();
        let mut buffer = HeapRb::new(48000 * 1);
        let (mut prod, mut cons) = buffer.split();
        let in_stream = build_input_stream(
            &device,
            config.clone(),
            move |audio_slice| match audio_slice {
                AudioSlice::F32(slice) => {
                    println!("Recording");
                    prod.push_slice(slice);
                }
                AudioSlice::I16(slice) => {
                    println!("Got {} i16 samples {:?}", slice.len(), &slice[0..5])
                }
                AudioSlice::I8(items) => todo!(),
                AudioSlice::I32(items) => todo!(),
                AudioSlice::I24(items) => {}
                AudioSlice::U24(items) => todo!(),
                AudioSlice::I64(items) => todo!(),
                AudioSlice::U64(items) => todo!(),
                AudioSlice::DsdU8(items) => todo!(),
                AudioSlice::DsdU16(items) => todo!(),
                AudioSlice::DsdU32(items) => todo!(),
            },
            |err| eprintln!("Stream error: {err}"),
        )
        .unwrap();

        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate() as f32;
        let channels: usize = config.channels().into();
        let out_stream = build_output_stream(
            &device,
            config.clone(),
            move |audio_slice| match audio_slice {
                AudioSliceMut::F32(slice) => {
                    println!("Playing");
                    for frame in slice.chunks_mut(channels) {
                        let mut values = [0.0; 2];
                        cons.pop_slice(&mut values);

                        println!("{:?}", values);

                        //Left channel modifier
                        let mut adjust = 1.1;
                        for (sample, val) in frame.iter_mut().zip(values.iter_mut()) {
                            *sample = *val * adjust;
                            //Right channel modifier
                            adjust = 4.0
                        }
                    }
                }
                _ => {
                    println!("Got ???");
                }
            },
            |err| eprintln!("Stream error: {err}"),
        )
        .unwrap();

        in_stream.play().unwrap();
        out_stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
        drop(in_stream);
        drop(out_stream);
    }

    #[derive(Debug, Default)]
    struct DispatchTester {
        data: Mutex<Vec<f32>>,
    }
    impl DispatchTester {
        fn new() -> Self {
            let data = Mutex::new(vec![]);
            Self { data }
        }
    }

    impl AudioSink for DispatchTester {
        fn on_read(&self, slice: AudioSlice<'_>) {
            let mut gaurd = self.data.lock().unwrap();
            match slice {
                AudioSlice::I8(items) => todo!(),
                AudioSlice::I16(items) => todo!(),
                AudioSlice::I32(items) => todo!(),
                AudioSlice::F32(items) => {
                    for f in items {
                        gaurd.push(*f);
                    }
                }
                AudioSlice::I24(items) => todo!(),
                AudioSlice::U24(items) => todo!(),
                AudioSlice::I64(items) => todo!(),
                AudioSlice::U64(items) => todo!(),
                AudioSlice::DsdU8(items) => todo!(),
                AudioSlice::DsdU16(items) => todo!(),
                AudioSlice::DsdU32(items) => todo!(),
            }
        }

        fn on_write(&self, slice: AudioSliceMut<'_>) {
            todo!()
        }
    }

    #[test]

    fn test_dispatcher() {
        println!("TESTING");
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();
        let mut in_stream = build_input_handle(&device, config.clone(), |err| {
            eprintln!("Stream error: {err}")
        })
        .unwrap();
        let test = Arc::new(DispatchTester::new());
        in_stream.dispatcher.push(test.clone());
        in_stream.stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        in_stream.stream.pause();
        let g = test.data.lock().unwrap();
        let d = g;
        println!("Data\n{:?}", d)
    }
}
