use std::io::Read;

use anyhow::Result;
use byteorder::ReadBytesExt;
use bytes::BufMut;
use serde::{Deserialize, Serialize};

use crate::planet::Planet;

#[cfg(not(target_arch = "wasm32"))]
pub const N_SAVE_FILES: usize = 99;
#[cfg(target_arch = "wasm32")]
pub const N_SAVE_FILES: usize = 1;

pub const SAVE_FILE_EXTENSION: &str = "planet";

const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SaveFileList(Vec<Option<SaveFile>>);

impl SaveFileList {
    pub fn saved_time(&self, i: usize) -> Option<&str> {
        self.0[i].as_ref().map(|save_file| save_file.time.as_ref())
    }
}

pub fn save_to(slot: usize, planet: &Planet, save_file_metadata: &SaveFileMetadata) -> Result<()> {
    let planet_data = rmp_serde::to_vec(planet)?;

    let mut save_file_metadata = save_file_metadata.clone();
    if slot != 0 {
        save_file_metadata.manual_slot = None;
    }

    log::info!("save to slot {}", slot);

    let bytes = SaveFile::new(planet_data, save_file_metadata.clone()).to_bytes();
    crate::platform::savefile_write(&format!("{:02}.{}", slot, SAVE_FILE_EXTENSION), &bytes)?;

    Ok(())
}

pub fn load_from(slot: usize) -> Result<(Planet, SaveFileMetadata)> {
    let data = SaveFile::from_bytes(&crate::platform::savefile_read(&format!(
        "{:02}.{}",
        slot, SAVE_FILE_EXTENSION
    ))?)?;
    log::info!(
        "load save from slot {} version={} time=\"{}\"",
        slot,
        data.version,
        data.time
    );
    let planet = rmp_serde::from_slice(&data.planet_data)?;
    let mut metadata = data.metadata;
    if metadata.manual_slot.is_none() && slot != 0 {
        metadata.manual_slot = Some(slot);
    }
    Ok((planet, metadata))
}

pub fn load_save_file_list() -> SaveFileList {
    SaveFileList(
        (0..=N_SAVE_FILES)
            .map(|i| {
                match crate::platform::savefile_read(&format!("{:02}.{}", i, SAVE_FILE_EXTENSION)) {
                    Ok(data) => match SaveFile::from_bytes(&data) {
                        Ok(save_file) => Some(save_file),
                        Err(e) => {
                            log::warn!("slot {} save data broken: {}", i, e);
                            None
                        }
                    },
                    Err(e) => {
                        if let Some(e) = e.downcast_ref::<std::io::Error>() {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                return None;
                            }
                        }
                        log::warn!("{}", e);
                        None
                    }
                }
            })
            .collect(),
    )
}

struct SaveFile {
    version: String,
    time: String,
    metadata: SaveFileMetadata,
    planet_data: Vec<u8>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SaveFileMetadata {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::unwrap_or_skip"
    )]
    pub manual_slot: Option<usize>,
    #[serde(default)]
    pub debug_mode_enabled: bool,
}

impl SaveFile {
    fn new(planet_data: Vec<u8>, metadata: SaveFileMetadata) -> Self {
        let time = chrono::Local::now().to_string();
        let time = time.split_once('.').unwrap().0.into(); // Get "YYYY-MM-DD hh:mm:ss"
        Self {
            version: GAME_VERSION.into(),
            time,
            metadata,
            planet_data,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let metadata = rmp_serde::to_vec_named(&self.metadata).unwrap();
        let mut buf = Vec::new();

        buf.put_u8(self.version.len().try_into().unwrap());
        buf.put(self.version.as_bytes());
        buf.put_u8(self.time.len().try_into().unwrap());
        buf.put(self.time.as_bytes());
        buf.put_u16(metadata.len().try_into().unwrap());
        buf.put(&metadata[..]);
        buf.put(&self.planet_data[..]);

        buf
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut data = std::io::Cursor::new(data);

        let len = data.read_u8()?;
        let mut version = vec![0; len as usize];
        data.read_exact(&mut version)?;
        let version = String::from_utf8(version)?;

        let len = data.read_u8()?;
        let mut time = vec![0; len as usize];
        data.read_exact(&mut time)?;
        let time = String::from_utf8(time)?;

        let len = data.read_u16::<byteorder::BigEndian>()?;
        let mut metadata = vec![0; len as usize];
        data.read_exact(&mut metadata)?;

        let mut planet_data = Vec::new();
        data.read_to_end(&mut planet_data)?;

        let metadata = match rmp_serde::from_slice(&metadata) {
            Ok(metadata) => metadata,
            Err(e) => {
                log::warn!("invalid metadata: {}", e);
                SaveFileMetadata::default()
            }
        };

        Ok(Self {
            version,
            time,
            metadata,
            planet_data,
        })
    }
}
