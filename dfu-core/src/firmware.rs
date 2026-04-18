use std::fs;
use std::path::Path;
use crate::error::DfuError;

#[derive(Debug)]
pub struct Firmware {
    pub data: Vec<u8>,
    pub path: String,
}

impl Firmware {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Firmware, DfuError> {
        let path_str = path.as_ref()
        .to_str()
        .unwrap_or("unknown")
        .to_string();

        let data = fs::read(&path)
        .map_err(|e| DfuError::InvalidFirmware(format!("could not read file: {}", e)))?;

        if data.is_empty(){
            return Err(DfuError::InvalidFirmware("file is empty".to_string()));
        }

        Ok(Firmware {
            data,
            path: path_str,
        })
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = &[u8]> {
        self.data.chunks(chunk_size)
    }
}