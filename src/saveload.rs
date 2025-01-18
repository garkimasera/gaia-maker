use std::io::Read;

use anyhow::{anyhow, Result};
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

pub fn save_to(slot: usize, planet: &Planet, manual_slot: usize) -> Result<()> {
    let planet_data = rmp_serde::to_vec(planet)?;

    log::info!("save to slot {}", slot);

    let metadata = SaveFileMetadata { manual_slot };
    let bytes = SaveFile::new(planet_data, metadata).to_bytes();
    write(&format!("{:02}.{}", slot, SAVE_FILE_EXTENSION), &bytes)?;

    Ok(())
}

pub fn load_from(slot: usize) -> Result<(Planet, usize)> {
    let data = SaveFile::from_bytes(&read(&format!("{:02}.{}", slot, SAVE_FILE_EXTENSION))?)?;
    log::info!(
        "load save from slot {} version={} time=\"{}\"",
        slot,
        data.version,
        data.time
    );
    let planet = rmp_serde::from_slice(&data.planet_data)?;
    Ok((planet, data.metadata.manual_slot))
}

pub fn load_save_file_list() -> SaveFileList {
    SaveFileList(
        (0..=N_SAVE_FILES)
            .map(
                |i| match read(&format!("{:02}.{}", i, SAVE_FILE_EXTENSION)) {
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
                },
            )
            .collect(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn write(file_name: &str, data: &[u8]) -> Result<()> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    std::fs::create_dir_all(&save_dir_path)?;
    std::fs::write(save_dir_path.join(file_name), data)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn read(file_name: &str) -> Result<Vec<u8>> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    Ok(std::fs::read(save_dir_path.join(file_name))?)
}

#[cfg(target_arch = "wasm32")]
fn write(file_name: &str, data: &[u8]) -> Result<()> {
    use std::io::Write;

    let base64_encoder =
        base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    let mut encoder = flate2::write::GzEncoder::new(base64_encoder, flate2::Compression::best());
    encoder.write_all(data)?;

    let s = encoder.finish()?.into_inner();

    log::info!("save {} bytes to local storage", s.len());

    crate::conf::get_storage()?
        .set_item(&format!("save/{}", file_name), &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn read(file_name: &str) -> Result<Vec<u8>> {
    use std::io::{Cursor, Read};

    let s = crate::conf::get_storage()?
        .get_item(&format!("save/{}", file_name))
        .map_err(|e| anyhow!("getItem failed: {:?}", e))?
        .ok_or_else(|| anyhow!("getItem failed"))?;
    let mut s = Cursor::new(s);
    let base64_decoder =
        base64::read::DecoderReader::new(&mut s, &base64::engine::general_purpose::STANDARD);
    let mut decoder = flate2::read::GzDecoder::new(base64_decoder);

    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;
    Ok(data)
}

struct SaveFile {
    version: String,
    time: String,
    metadata: SaveFileMetadata,
    planet_data: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct SaveFileMetadata {
    #[serde(default)]
    manual_slot: usize,
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
                log::warn!("invalid meatadata {}", e);
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
