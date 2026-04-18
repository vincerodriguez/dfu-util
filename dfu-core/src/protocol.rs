use crate::error::DfuError;
use crate::state_machine::{DfuState, DfuStatus};

pub const DFU_DETACH: u8    = 0;
pub const DFU_DNLOAD: u8    = 1;
pub const DFU_UPLOAD: u8    = 2;
pub const DFU_GETSTATUS: u8 = 3;
pub const DFU_CLRSTATUS: u8 = 4;
pub const DFU_GETSTATE: u8  = 5;
pub const DFU_ABORT: u8     = 6;

pub const USB_CLASS_APP_SPECIFIC: u8 = 0xFE;
pub const USB_SUBCLASS_DFU: u8       = 0x01;

#[derive(Debug, Clone)]
pub struct DfuStatusResponse {
    pub status: DfuStatus,
    pub poll_timeout_ms: u32,
    pub state: DfuState,
    pub string_index: u8,
}

impl DfuStatusResponse {
    pub fn from_bytes(bytes: &[u8]) -> Result<DfuStatusResponse, DfuError> {
        if bytes.len() < 6 {
            return Err(DfuError::Protocol(format!(
                "status response too short: got {} bytes, expected 6",
                bytes.len()
            )));
        }

        let status = DfuStatus::from_u8(bytes[0])
            .ok_or_else(|| DfuError::Protocol(format!("unknown status byte: {:#04x}", bytes[0])))?;

        let poll_timeout_ms = (bytes[1] as u32)
            | ((bytes[2] as u32) << 8)
            | ((bytes[3] as u32) << 16);

        let state = DfuState::from_u8(bytes[4])
            .ok_or_else(|| DfuError::Protocol(format!("unknown state byte: {:#04x}", bytes[4])))?;

        Ok(DfuStatusResponse {
            status,
            poll_timeout_ms,
            state,
            string_index: bytes[5],
        })
    }
}