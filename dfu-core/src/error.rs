use thiserror::Error;

#[derive(Debug, Error)]
pub enum DfuError {
    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),

    #[error("device not found")]
    DeviceNotFound,

    #[error("invalid firmware file: {0}")]
    InvalidFirmware(String),

    #[error("DFU protocol error: {0}")]
    Protocol(String),

    #[error("transfer failed at block {block}: {reason}")]
    TransferFailed { block: u32, reason: String },

    #[error("device is in error state: {0}")]
    DeviceError(String),   
}