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
    progress_cb: impl Fn(TransferProgress),
) -> Result<(), DfuError> {
    info!("starting download of {} bytes from {}", firmware.size(), firmware.path);

    enter_dfu_mode(handle)?;

    let mut block_num: u16 = 0;
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
    handle.download_block(block_num, &[])?;

    wait_for_manifest(handle)?;

    info!("download complete");
    Ok(())
}

fn enter_dfu_mode(handle: &DfuHandle) -> Result<(), DfuError> {
    let status = handle.get_status()?;

    match status.state {
        DfuState::DfuIdle => {
            debug!("device already in DFU idle state");
            Ok(())
        }
        DfuState::DfuError => {
            warn!("device in error state, clearing status");
            handle.clear_status()?;
            let status = handle.get_status()?;
            if status.state != DfuState::DfuIdle {
                return Err(DfuError::Protocol(
                    "device did not return to idle after clear".to_string()
                ));
            }
            Ok(())
        }
        DfuState::AppIdle => {
            info!("device in app mode, sending detach");
            handle.detach()?;
            Err(DfuError::Protocol(
                "device must re-enumerate as DFU device after detach, please re-run".to_string()
            ))
        }
        state => {
            handle.abort()?;
            Err(DfuError::Protocol(format!(
                "unexpected state at start of download: {:?}", state
            )))
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
                return Err(DfuError::DeviceError(
                    format!("device entered error state during manifest: {:?}", status.status)
                ));
            }
            state => {
                return Err(DfuError::Protocol(format!(
                    "unexpected state during manifest: {:?}", state
                )));
            }
        }
    }
}
