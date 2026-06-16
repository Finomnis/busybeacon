use std::collections::HashMap;

use async_hid::{AsyncHidFeatureHandle, AsyncHidRead, AsyncHidWrite, HidBackend, HidError};
use futures::StreamExt;
use miette::Diagnostic;
use thiserror::Error;

pub const VID: u16 = 0x1209;
pub const PID: u16 = 0xd9d0;

#[derive(Error, Diagnostic, Debug)]
pub enum BusyBeaconError {
    #[error(transparent)]
    #[diagnostic(code(busybeacon::hid))]
    IoError(#[from] HidError),

    #[error("Device not found")]
    #[diagnostic(code(busybeacon::no_device))]
    DeviceNotFound,

    #[error("Device reported an unexpected state")]
    #[diagnostic(code(busybeacon::unexpected_state))]
    UnexpectedDeviceState,

    #[error("Device responded with an invalid feature report")]
    #[diagnostic(code(busybeacon::invalid_feature_report))]
    InvalidFeatureReport,

    #[error("Device sent an invalid input report")]
    #[diagnostic(code(busybeacon::invalid_input_report))]
    InvalidInputReport,
}

pub struct BusyBeacon {
    reader: async_hid::DeviceReader,
    writer: async_hid::DeviceWriter,
    feature: async_hid::DeviceFeatureHandle,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BusyBeaconState {
    Off = 0,
    Green = 1,
    Yellow = 2,
    Red = 3,
}

impl TryFrom<u8> for BusyBeaconState {
    type Error = BusyBeaconError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Green),
            2 => Ok(Self::Yellow),
            3 => Ok(Self::Red),
            _ => Err(BusyBeaconError::UnexpectedDeviceState),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BusyBeaconDeviceInfo {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub version: Option<u16>,
}

impl BusyBeacon {
    pub async fn new() -> Result<Self, BusyBeaconError> {
        let backend = HidBackend::default();
        let mut devices = backend.enumerate().await?;

        let device = loop {
            match devices.next().await {
                Some(dev) if dev.vendor_id == VID && dev.product_id == PID => {
                    break dev;
                }
                Some(_) => continue,
                None => return Err(BusyBeaconError::DeviceNotFound),
            }
        };

        let (reader, writer) = device.open().await?;

        let feature = device.open_feature_handle().await?;

        Ok(Self {
            reader,
            writer,
            feature,
        })
    }

    pub async fn new_with_serial(serial: impl AsRef<str>) -> Result<Self, BusyBeaconError> {
        let serial = serial.as_ref();

        let backend = HidBackend::default();
        let mut devices = backend.enumerate().await?;

        let device = loop {
            match devices.next().await {
                Some(dev)
                    if dev.vendor_id == VID
                        && dev.product_id == PID
                        && dev.serial_number.as_deref() == Some(serial) =>
                {
                    break dev;
                }
                Some(_) => continue,
                None => return Err(BusyBeaconError::DeviceNotFound),
            }
        };

        let (reader, writer) = device.open().await?;

        let feature = device.open_feature_handle().await?;

        Ok(Self {
            reader,
            writer,
            feature,
        })
    }

    pub async fn list_devices() -> Result<Vec<BusyBeaconDeviceInfo>, BusyBeaconError> {
        let versions = nusb::list_devices().await.ok().map(|devs| {
            devs.filter_map(|dev| {
                if dev.vendor_id() == VID && dev.product_id() == PID {
                    dev.serial_number()
                        .map(|serial| (serial.to_string(), dev.device_version()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>()
        });

        let devices = HidBackend::default()
            .enumerate()
            .await?
            .filter_map(async |dev| {
                if dev.vendor_id != VID || dev.product_id != PID {
                    return None;
                }

                let version =
                    if let (Some(versions), Some(serial)) = (&versions, &dev.serial_number) {
                        versions.get(serial).copied()
                    } else {
                        None
                    };

                Some(BusyBeaconDeviceInfo {
                    name: dev.name.clone(),
                    vendor_id: dev.vendor_id,
                    product_id: dev.product_id,
                    serial_number: dev.serial_number.clone(),
                    version,
                })
            })
            .collect::<Vec<_>>()
            .await;

        Ok(devices)
    }

    async fn send_value(&mut self, value: u8) -> Result<(), BusyBeaconError> {
        self.writer
            .write_output_report(&[0x01, value])
            .await
            .map_err(Into::into)
    }

    pub async fn set_state(&mut self, state: BusyBeaconState) -> Result<(), BusyBeaconError> {
        self.send_value(state as u8).await
    }

    pub async fn read_state(&mut self) -> Result<BusyBeaconState, BusyBeaconError> {
        let mut buf = [0x02, 0x00];
        let read_len = self.feature.read_feature_report(&mut buf).await?;

        match read_len {
            // Backends that return report ID + one-byte payload.
            2 => BusyBeaconState::try_from(buf[1]),

            // Backends that return only the one-byte payload.
            1 => BusyBeaconState::try_from(buf[0]),

            _ => Err(BusyBeaconError::InvalidFeatureReport),
        }
    }

    pub async fn wait_for_state_change(&mut self) -> Result<BusyBeaconState, BusyBeaconError> {
        let mut buf = [0u8; 2];
        let read_len = self.reader.read_input_report(&mut buf).await?;

        if read_len != 2 || buf[0] != 0x03 {
            Err(BusyBeaconError::InvalidInputReport)
        } else {
            BusyBeaconState::try_from(buf[1])
        }
    }
}
