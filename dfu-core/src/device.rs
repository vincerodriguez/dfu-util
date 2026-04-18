
use rusb::{Context, DeviceHandle, UsbContext};
use crate::error::DfuError;
use crate::protocol::{
    DfuStatusResponse,
    DFU_GETSTATUS,
    DFU_GETSTATE,
    DFU_CLRSTATUS,
    DFU_ABORT,
    DFU_DETACH,
    DFU_DNLOAD,
};
use crate::state_machine::DfuState;

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(5000);

const REQUEST_TYPE_OUT: u8 = 0x21;
const REQUEST_TYPE_IN: u8  = 0xA1;

pub struct DfuHandle {
    handle: DeviceHandle<Context>,
    pub interface: u8,
}

impl DfuHandle {
    pub fn open(vendor_id: u16, product_id: u16) -> Result<DfuHandle, DfuError> {
        let context = Context::new()?;

        let handle = context
            .open_device_with_vid_pid(vendor_id, product_id)
            .ok_or(DfuError::DeviceNotFound)?;

        let interface = 0;
        handle.claim_interface(interface)?;

        Ok(DfuHandle { handle, interface })
    }

    pub fn get_status(&self) -> Result<DfuStatusResponse, DfuError> {
        let mut buf = [0u8; 6];

        self.handle.read_control(
            REQUEST_TYPE_IN,
            DFU_GETSTATUS,
            0,
            self.interface as u16,
            &mut buf,
            TIMEOUT,
        )?;

        DfuStatusResponse::from_bytes(&buf)
    }

    pub fn get_state(&self) -> Result<DfuState, DfuError> {
        let mut buf = [0u8; 1];

        self.handle.read_control(
            REQUEST_TYPE_IN,
            DFU_GETSTATE,
            0,
            self.interface as u16,
            &mut buf,
            TIMEOUT,
        )?;

        DfuState::from_u8(buf[0])
            .ok_or_else(|| DfuError::Protocol(format!("unknown state byte: {:#04x}", buf[0])))
    }

    pub fn clear_status(&self) -> Result<(), DfuError> {
        self.handle.write_control(
            REQUEST_TYPE_OUT,
            DFU_CLRSTATUS,
            0,
            self.interface as u16,
            &[],
            TIMEOUT,
        )?;

        Ok(())
    }

    pub fn abort(&self) -> Result<(), DfuError> {
        self.handle.write_control(
            REQUEST_TYPE_OUT,
            DFU_ABORT,
            0,
            self.interface as u16,
            &[],
            TIMEOUT,
        )?;

        Ok(())
    }

    pub fn detach(&self) -> Result<(), DfuError> {
        self.handle.write_control(
            REQUEST_TYPE_OUT,
            DFU_DETACH,
            0,
            self.interface as u16,
            &[],
            TIMEOUT,
        )?;

        Ok(())
    }

    pub fn download_block(
        &self,
        block_num: u16,
        data: &[u8],
    ) -> Result<(), DfuError> {
        self.handle.write_control(
            REQUEST_TYPE_OUT,
            DFU_DNLOAD,
            block_num,
            self.interface as u16,
            data,
            TIMEOUT,
        )?;

        Ok(())
    }
}

impl Drop for DfuHandle {
    fn drop(&mut self) {
        let _ = self.handle.release_interface(self.interface);
    }
}
impl DfuHandle {
    pub fn set_address(&self, address: u32) -> Result<(), DfuError> {
        let cmd = [
            crate::protocol::STM32_CMD_SET_ADDRESS,
            (address & 0xFF) as u8,
            ((address >> 8) & 0xFF) as u8,
            ((address >> 16) & 0xFF) as u8,
            ((address >> 24) & 0xFF) as u8,
        ];

        self.download_block(0, &cmd)?;
        self.wait_for_idle_after_command()
    }

    pub fn mass_erase(&self) -> Result<(), DfuError> {
        let cmd = [crate::protocol::STM32_CMD_ERASE_ALL];

        self.download_block(0, &cmd)?;
        self.wait_for_idle_after_command()
    }

    fn wait_for_idle_after_command(&self) -> Result<(), DfuError> {
        loop {
            let status = self.get_status()?;
            match status.state {
                crate::state_machine::DfuState::DfuDownloadBusy => {
                    std::thread::sleep(std::time::Duration::from_millis(
                        status.poll_timeout_ms as u64
                    ));
                }
                crate::state_machine::DfuState::DfuDownloadIdle => return Ok(()),
                crate::state_machine::DfuState::DfuError => {
                    return Err(DfuError::Protocol(format!(
                        "device error after command: {:?}", status.status
                    )));
                }
                state => {
                    return Err(DfuError::Protocol(format!(
                        "unexpected state after command: {:?}", state
                    )));
                }
            }
        }
    }
}
