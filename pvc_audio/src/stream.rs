use cpal::{Device, SampleFormat, Stream, StreamConfig, StreamError, SupportedStreamConfig, traits::DeviceTrait};

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
