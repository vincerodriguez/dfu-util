use log::{info, warn, debug};
use crate::error::DfuError;
use crate::device::DfuHandle;
use crate::firmware::Firmware;
use crate::state_machine::{DfuState, DfuStatus};

const DFU_BLOCK_SIZE: usize = 2048;

pub struct TransferProgress {
    pub bytes_sent: usize,
    pub total_bytes: usize,
    pub block_num: u16,
}

impl TransferProgress {
    pub fn percent(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.bytes_sent as f32 / self.total_bytes as f32) * 100.0
    }
}

pub fn download(
    handle: &DfuHandle,
    firmware: &Firmware,
    start_address: u32,
    mass_erase: bool,
    progress_cb: impl Fn(TransferProgress),
) -> Result<(), DfuError> {
    info!("starting download of {} bytes from {}", firmware.size(), firmware.path);

    enter_dfu_mode(handle)?;

    if mass_erase {
        info!("erasing flash — this may take a few seconds");
        handle.mass_erase()?;
        info!("erase complete");
    }

    info!("setting start address to {:#010x}", start_address);
    handle.set_address(start_address)?;

    handle.abort()?;
    let status = handle.get_status()?;
    if status.state != DfuState::DfuIdle {
        return Err(DfuError::Protocol(format!(
            "expected DfuIdle after abort, got {:?}", status.state
        )));
    }

    let mut block_num: u16 = 2;
    let mut bytes_sent: usize = 0;
    let total_bytes = firmware.size();

    for chunk in firmware.chunks(DFU_BLOCK_SIZE) {
        debug!("sending block {} ({} bytes)", block_num, chunk.len());

        handle.download_block(block_num, chunk)?;

        wait_for_idle(handle, block_num)?;

        bytes_sent += chunk.len();
        block_num += 1;

        progress_cb(TransferProgress {
            bytes_sent,
            total_bytes,
            block_num,
        });
    }

    info!("sending zero-length block to signal end of firmware");
    match handle.download_block(block_num, &[]) {
        Ok(_) => {}
        Err(DfuError::Usb(rusb::Error::Io)) => {
            info!("device reset after zero-length block — this is normal for STM32");
            return Ok(());
        }
        Err(e) => return Err(e),
    }

    match wait_for_manifest(handle) {
        Ok(_) => {}
        Err(DfuError::Usb(rusb::Error::Io)) | Err(DfuError::Usb(rusb::Error::NoDevice)) => {
            info!("device disconnected during manifest — normal, device is rebooting");
        }
        Err(e) => return Err(e),
    }

    info!("download complete");
    Ok(())
}

fn enter_dfu_mode(handle: &DfuHandle) -> Result<(), DfuError> {
    let status = handle.get_status()?;

    info!("device state on entry: {:?}", status.state);

    match status.state {
        DfuState::DfuIdle => {
            debug!("device already in DFU idle state");
            Ok(())
        }
        DfuState::DfuError => {
            warn!("device in error state, clearing");
            handle.clear_status()?;
            let status = handle.get_status()?;
            if status.state != DfuState::DfuIdle {
                return Err(DfuError::Protocol(
                    "device did not return to idle after clear".to_string()
                ));
            }
            Ok(())
        }
        DfuState::DfuDownloadIdle | DfuState::DfuDownloadSync => {
            warn!("device in stale download state, aborting");
            handle.abort()?;
            let status = handle.get_status()?;
            if status.state == DfuState::DfuError {
                handle.clear_status()?;
            }
            let status = handle.get_status()?;
            if status.state != DfuState::DfuIdle {
                return Err(DfuError::Protocol(format!(
                    "could not recover to idle, stuck in {:?}", status.state
                )));
            }
            Ok(())
        }
        DfuState::AppIdle => {
            info!("device in app mode, sending detach");
            handle.detach()?;
            Err(DfuError::Protocol(
                "device must re-enumerate after detach, please re-run".to_string()
            ))
        }
        state => {
            warn!("unexpected state {:?}, attempting abort + clear", state);
            let _ = handle.abort();
            let _ = handle.clear_status();
            let status = handle.get_status()?;
            if status.state != DfuState::DfuIdle {
                return Err(DfuError::Protocol(format!(
                    "could not recover to idle from {:?}", state
                )));
            }
            Ok(())
        }
    }
}

fn wait_for_idle(handle: &DfuHandle, block_num: u16) -> Result<(), DfuError> {
    loop {
        let status = handle.get_status()?;

        match status.state {
            DfuState::DfuDownloadIdle => return Ok(()),
            DfuState::DfuDownloadBusy => {
                debug!("device busy, waiting {}ms", status.poll_timeout_ms);
                std::thread::sleep(std::time::Duration::from_millis(
                    status.poll_timeout_ms as u64
                ));
            }
            DfuState::DfuError => {
                return Err(DfuError::TransferFailed {
                    block: block_num as u32,
                    reason: format!("device entered error state: {:?}", status.status),
                });
            }
            state => {
                return Err(DfuError::Protocol(format!(
                    "unexpected state during download: {:?}", state
                )));
            }
        }

        if status.status != DfuStatus::Ok {
            return Err(DfuError::TransferFailed {
                block: block_num as u32,
                reason: format!("bad status: {:?}", status.status),
            });
        }
    }
}

fn wait_for_manifest(handle: &DfuHandle) -> Result<(), DfuError> {
    loop {
        let status = handle.get_status()?;

        match status.state {
            DfuState::DfuManifest | DfuState::DfuManifestSync => {
                debug!("device manifesting, waiting {}ms", status.poll_timeout_ms);
                std::thread::sleep(std::time::Duration::from_millis(
                    status.poll_timeout_ms as u64
                ));
            }
            DfuState::DfuManifestWaitReset | DfuState::DfuIdle => {
                return Ok(());
            }
            DfuState::DfuError => {
                return Err(DfuError::DeviceError(format!(
                    "device entered error state during manifest: {:?}", status.status
                )));
            }
            state => {
                return Err(DfuError::Protocol(format!(
                    "unexpected state during manifest: {:?}", state
                )));
            }
        }
    }
}
