use std::fmt;

use cpal::{
    Device, Host, SampleFormat, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait},
};

use crate::{Result, error::AudioError};

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub bits_per_sample: u32,
}

#[derive(Debug, Clone)]
pub enum DeviceLevel {
    Hardware,
    System,
    Alias,
}

pub fn bits_per_sample(format: SampleFormat) -> u32 {
    match format {
        SampleFormat::I8 | SampleFormat::U8 => 8,

        SampleFormat::I16 | SampleFormat::U16 => 16,

        SampleFormat::I24 | SampleFormat::U24 => 24,

        SampleFormat::I32 | SampleFormat::U32 | SampleFormat::F32 => 32,

        SampleFormat::I64 | SampleFormat::U64 | SampleFormat::F64 => 64,

        SampleFormat::DsdU8 => 8,
        SampleFormat::DsdU16 => 16,
        SampleFormat::DsdU32 => 32,
        _ => 32,
    }
}

#[derive(Clone)]
pub struct AudioDevice<DeviceType> {
    pub devtype: DeviceType,
    pub device: cpal::Device,
    pub info: AudioDeviceInfo,
    pub level: DeviceLevel,
    pub config: SupportedStreamConfig,
}

#[derive(Debug, Default, Clone)]
pub struct Input;
#[derive(Debug, Default, Clone)]
pub struct Output;

impl fmt::Debug for AudioDevice<Input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("[INPUT]AudioDevice")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for AudioDevice<Output> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("[OUTPUT]AudioDevice")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

fn get_device_configs(device: &Device) -> Result<SupportedStreamConfig> {
    let config_range = device
        .supported_input_configs()?
        .next()
        .ok_or(AudioError::NoInfoForAudioDevice)?;
    let config = config_range.with_max_sample_rate();

    return Ok(config);
}

fn get_input_devices(host: &Host) -> Result<Vec<AudioDevice<Input>>> {
    let mut devices: Vec<AudioDevice<Input>> = vec![];
    for device in host.input_devices()? {
        let config = match get_device_configs(&device) {
            Ok(conf) => conf,
            Err(e) => {
                continue;
            }
        };
        let sample_format = config.sample_format();
        let info = AudioDeviceInfo {
            id: device.id()?.1,
            name: device.description()?.name().into(),
            bits_per_sample: bits_per_sample(sample_format),
        };

        devices.push(AudioDevice {
            devtype: Input,
            device,
            info,
            level: DeviceLevel::Hardware,
            config,
        });
    }
    Ok(devices)
}

fn get_output_devices(host: &Host) -> Result<Vec<AudioDevice<Output>>> {
    let mut devices: Vec<AudioDevice<Output>> = vec![];
    for device in host.output_devices()? {
        let config = match get_device_configs(&device) {
            Ok(conf) => conf,
            Err(e) => {
                continue;
            }
        };
        let sample_format = config.sample_format();
        let info = AudioDeviceInfo {
            id: device.id()?.1,
            name: device.description()?.name().into(),
            bits_per_sample: bits_per_sample(sample_format),
        };

        devices.push(AudioDevice {
            devtype: Output,
            device,
            info,
            level: DeviceLevel::Hardware,
            config,
        });
    }
    Ok(devices)
}

fn is_noise_device(name: &str, id: &str) -> bool {
    let bad_prefixes = [
        "hw:",
        "plughw:",
        "sysdefault:",
        "front:",
        "surround",
        "iec958",
    ];

    let bad_names = ["Discard all samples"];

    bad_names.iter().any(|n| name.contains(n)) || bad_prefixes.iter().any(|p| id.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_devices() {
        let host = cpal::default_host();
        let devices = get_input_devices(&host);

        match devices {
            Ok(dev) => {
                // let dev = filter_devices(dev);
                for d in dev {
                    println!("{:?}", d);
                    println!("Channels: {}", d.config.channels());
                }
            }
            Err(e) => println!("error: {:?}", e),
        }
    }
}
