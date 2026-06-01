use cpal::{
    Device, SampleFormat, Stream, StreamConfig, StreamError, SupportedStreamConfig,
    traits::DeviceTrait,
};

use crate::{Result, error::AudioError};

pub struct InputStream {
    stream: cpal::Stream,
}

pub struct OutputStream {
    stream: cpal::Stream,
}

pub enum AudioSlice<'a> {
    I8(&'a [i8]),
    I16(&'a [i16]),
    I32(&'a [i32]),
    F32(&'a [f32]),
}

pub fn build_input_stream<D, E>(
    device: &Device,
    config: SupportedStreamConfig,
    mut on_data: D,
    on_err: E,
) -> Result<Stream>
where
    D: FnMut(AudioSlice<'_>) + Send + 'static,
    E: FnMut(StreamError) + Send + 'static,
{
    let stream_config: StreamConfig = config.clone().into();

    let stream = match config.sample_format() {
        SampleFormat::I8 => device.build_input_stream(
            &stream_config,
            move |data: &[i8], _| on_data(AudioSlice::I8(data)),
            on_err,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| on_data(AudioSlice::I16(data)),
            on_err,
            None,
        )?,
        SampleFormat::I32 => device.build_input_stream(
            &stream_config,
            move |data: &[i32], _| on_data(AudioSlice::I32(data)),
            on_err,
            None,
        )?,
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| on_data(AudioSlice::F32(data)),
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
    D: FnMut(AudioSlice<'_>) + Send + 'static,
    E: FnMut(StreamError) + Send + 'static,
{
    let stream_config: StreamConfig = config.clone().into();

    let stream = match config.sample_format() {
        SampleFormat::I8 => device.build_output_stream(
            &stream_config,
            move |data: &mut [i8], _| on_data(AudioSlice::I8(data)),
            on_err,
            None,
        )?,
        SampleFormat::I16 => device.build_output_stream(
            &stream_config,
            move |data: &mut [i16], _| on_data(AudioSlice::I16(data)),
            on_err,
            None,
        )?,
        SampleFormat::I32 => device.build_output_stream(
            &stream_config,
            move |data: &mut [i32], _| on_data(AudioSlice::I32(data)),
            on_err,
            None,
        )?,
        SampleFormat::F32 => device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _| on_data(AudioSlice::F32(data)),
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
#[cfg(test)] // Compiles this module ONLY when running 'cargo test'
mod tests {
    use cpal::traits::{HostTrait, StreamTrait};

    use super::*; // Brings the outer functions into scope

    //Physcial mic test not to be used in unit tests
    #[test] // Marks this specific function as a runnable test
    #[ignore]
    fn test_input_audio() {
        println!("TESTING");
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();

        // Look at that beautiful dot-notation!
        let stream = build_input_stream(
            &device,
            config.clone(),
            |audio_slice| {
                // The user handles data dynamically without knowing types ahead of time
                match audio_slice {
                    AudioSlice::F32(slice) => {
                        println!("Got {} f32 samples {:?}", slice.len(), &slice[0..5])
                    }
                    AudioSlice::I16(slice) => {
                        println!("Got {} i16 samples {:?}", slice.len(), &slice[0..5])
                    }
                    _ => {
                        println!("Got ???");
                    }
                }
            },
            |err| eprintln!("Stream error: {err}"),
        )
        .unwrap();

        stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(stream);
    }
}
