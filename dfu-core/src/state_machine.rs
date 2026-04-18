#[derive(Debug, Clone, PartialEq)]
pub enum DfuState {
    AppIdle, 
    AppDetach, 
    DfuIdle, 
    DfuDownloadSync,
    DfuDownloadBusy,
    DfuDownloadIdle,
    DfuManifestSync,
    DfuManifest,
    DfuManifestWaitReset,
    DfuUploadIdle,
    DfuError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DfuStatus {
    Ok, 
    ErrTarget,
    ErrFile, 
    ErrWrite, 
    ErrErase, 
    ErrCheckErased,
    ErrProg, 
    ErrVerify,
    ErrAddress,
    ErrNotDone,
    ErrFirmware,
    ErrVendor,
    ErrUsbR,
    ErrPor,
    ErrUnknown,
    ErrStaledPkt,
}

impl DfuState {
    pub fn from_u8(value: u8) -> Option<DfuState>{
        match value {
            0 => Some(DfuState::AppIdle),
            1 => Some(DfuState::AppDetach),
            2 => Some(DfuState::DfuIdle),
            3 => Some(DfuState::DfuDownloadSync),
            4 => Some(DfuState::DfuDownloadBusy),
            5 => Some(DfuState::DfuDownloadIdle),
            6 => Some(DfuState::DfuManifestSync),
            7 => Some(DfuState::DfuManifest),
            8 => Some(DfuState::DfuManifestWaitReset),
            9 => Some(DfuState::DfuUploadIdle),
            10 => Some(DfuState::DfuError),
            _ => None,            
        }
    }
    pub fn is_dfu_mode(&self) -> bool {
        match self {
            DfuState::AppIdle | DfuState::AppDetach => false,
            _ => true,
        }
    }    
}

impl DfuStatus {
    pub fn from_u8(value: u8) -> Option<DfuStatus> {
        match value {
            0x00 => Some(DfuStatus::Ok),
            0x01 => Some(DfuStatus::ErrTarget),
            0x02 => Some(DfuStatus::ErrFile),
            0x03 => Some(DfuStatus::ErrWrite),
            0x04 => Some(DfuStatus::ErrErase),
            0x05 => Some(DfuStatus::ErrCheckErased),
            0x06 => Some(DfuStatus::ErrProg),
            0x07 => Some(DfuStatus::ErrVerify),
            0x08 => Some(DfuStatus::ErrAddress),
            0x09 => Some(DfuStatus::ErrNotDone),
            0x0A => Some(DfuStatus::ErrFirmware),
            0x0B => Some(DfuStatus::ErrVendor),
            0x0C => Some(DfuStatus::ErrUsbR),
            0x0D => Some(DfuStatus::ErrPor),
            0x0E => Some(DfuStatus::ErrUnknown),
            0x0F => Some(DfuStatus::ErrStaledPkt),
            _ => None,
        }
    }
}