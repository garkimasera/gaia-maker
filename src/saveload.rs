use std::{collections::VecDeque, io::Read};

use anyhow::Result;
use byteorder::ReadBytesExt;
use bytes::BufMut;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::planet::Planet;

pub const SAVE_FILE_EXTENSION: &str = "planet";

const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SavedTime(String);

#[derive(Default, Debug)]
pub struct SaveState {
    pub current: String,
    pub list: VecDeque<(SavedTime, String)>, // Latest saved time and name list
    pub manual_save_files: Vec<u32>,
    pub auto_save_files: Vec<u32>,
    pub save_file_metadata: SaveFileMetadata,
}

pub fn save_to(planet: &Planet, save_state: &mut SaveState, auto: bool) -> Result<()> {
    let planet_data = rmp_serde::to_vec(planet)?;

    let bytes = SaveFile::new(
        planet_data,
        &planet.basics.name,
        save_state.save_file_metadata.clone(),
    )
    .to_bytes();

    let file_name = if auto {
        let n = save_state
            .auto_save_files
            .iter()
            .copied()
            .max()
            .unwrap_or_default()
            + 1;
        save_state.auto_save_files.push(n);
        format!("autosave{:06}.{}", n, SAVE_FILE_EXTENSION)
    } else {
        let n = save_state
            .manual_save_files
            .iter()
            .copied()
            .max()
            .unwrap_or_default()
            + 1;
        save_state.manual_save_files.push(n);
        format!("{:06}.{}", n, SAVE_FILE_EXTENSION)
    };

    log::info!("save {}/{}", save_state.current, file_name);

    crate::platform::write_savefile(&save_state.current, &file_name, &bytes)?;

    Ok(())
}

pub fn load_from(save_state: &SaveState, auto: bool, n: u32) -> Result<(Planet, SaveFileMetadata)> {
    let file_name = if auto {
        format!("autosave{:06}.{}", n, SAVE_FILE_EXTENSION)
    } else {
        format!("{:06}.{}", n, SAVE_FILE_EXTENSION)
    };
    let data = SaveFile::from_bytes(&crate::platform::read_savefile(
        &save_state.current,
        &file_name,
    )?)?;
    log::info!(
        "load save from {} version={} time=\"{}\"",
        file_name,
        data.version,
        data.time.0
    );
    let planet = rmp_serde::from_slice(&data.planet_data)?;
    Ok((planet, data.metadata))
}

pub struct SaveFile {
    version: String,
    time: SavedTime,
    name: String,
    metadata: SaveFileMetadata,
    planet_data: Vec<u8>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SaveFileMetadata {
    #[serde(default)]
    pub debug_mode_enabled: bool,
}

impl SaveFile {
    fn new(planet_data: Vec<u8>, name: &str, metadata: SaveFileMetadata) -> Self {
        Self {
            version: GAME_VERSION.into(),
            time: SavedTime::now(),
            name: name.into(),
            metadata,
            planet_data,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let metadata = rmp_serde::to_vec_named(&self.metadata).unwrap();
        let mut buf = Vec::new();

        buf.put_u8(self.version.len().try_into().unwrap());
        buf.put(self.version.as_bytes());
        buf.put_u8(self.time.0.len().try_into().unwrap());
        buf.put(self.time.0.as_bytes());
        buf.put_u8(self.name.len().try_into().unwrap());
        buf.put(self.name.as_bytes());
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
        let time = SavedTime(String::from_utf8(time)?);

        let len = data.read_u8()?;
        let mut name = vec![0; len as usize];
        data.read_exact(&mut name)?;
        let name = String::from_utf8(name)?;

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
            name,
            metadata,
            planet_data,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SaveSubDirItem {
    pub time: SavedTime,
    pub _file_name: String,
    pub auto: bool,
    pub n: u32,
}

static RE_SAVE_FILE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"(autosave)?(\d+)\..+").unwrap());

pub fn read_save_sub_dir(sub_dir_name: &str) -> Vec<SaveSubDirItem> {
    let mut list = Vec::new();

    let save_sub_dir_files = match crate::platform::save_sub_dir_files(sub_dir_name) {
        Ok(save_sub_dir_files) => save_sub_dir_files,
        Err(e) => {
            log::warn!("{}", e);
            return Vec::new();
        }
    };

    let ext = format!(".{}", SAVE_FILE_EXTENSION);
    for file_name in save_sub_dir_files {
        if !file_name.ends_with(&ext) {
            continue;
        }
        let save_file_data = match crate::platform::read_savefile(sub_dir_name, &file_name) {
            Ok(save_file_data) => save_file_data,
            Err(e) => {
                log::warn!("cannot read {}: {:?}", file_name, e);
                continue;
            }
        };
        let save_file = match SaveFile::from_bytes(&save_file_data) {
            Ok(save_file) => save_file,
            Err(e) => {
                log::warn!("cannot load {}: {:?}", file_name, e);
                continue;
            }
        };

        let Some(caps) = RE_SAVE_FILE.captures(&file_name) else {
            log::warn!("invalid save file name: {}", file_name);
            continue;
        };

        let auto = caps
            .get(1)
            .map(|prefix| prefix.as_str() == "autosave")
            .unwrap_or_default();
        let n: u32 = caps.get(2).unwrap().as_str().parse().unwrap();

        list.push(SaveSubDirItem {
            time: save_file.time,
            _file_name: file_name,
            auto,
            n,
        });
    }

    list.sort_by_key(|item| std::cmp::Reverse(item.time.clone()));
    list
}

impl SavedTime {
    pub fn now() -> Self {
        let time = chrono::Local::now().to_string();
        Self(time.split_once('.').unwrap().0.into()) // Get "YYYY-MM-DD hh:mm:ss"
    }
}

impl std::fmt::Display for SavedTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<std::time::SystemTime> for SavedTime {
    fn from(value: std::time::SystemTime) -> Self {
        let time: chrono::DateTime<chrono::Local> = value.into();
        Self(time.to_string().split_once('.').unwrap().0.into())
    }
}
