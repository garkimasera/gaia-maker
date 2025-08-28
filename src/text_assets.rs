use anyhow::bail;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use crossbeam::atomic::AtomicCell;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use strum::{AsRefStr, EnumIter};

#[derive(Clone, Copy, Debug)]
pub struct TextAssetsPlugin;

impl Plugin for TextAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TranslationText>()
            .register_asset_loader(TextLoader)
            .init_asset::<RandomNameList>()
            .register_asset_loader(RandomNameListLoader);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, EnumIter, AsRefStr)]
pub enum Lang {
    #[strum(serialize = "en")]
    English,
    #[strum(serialize = "ja")]
    Japanese,
}

impl Lang {
    pub fn name(&self) -> &'static str {
        match self {
            Lang::English => "English",
            Lang::Japanese => "日本語",
        }
    }
}

macro_rules! t {
    ($s:expr) => {
        $crate::text_assets::get_text(&$s, std::collections::HashMap::default())
    };
    ($s:expr; $($name:ident = $value:expr),*) => {{
        let mut map = std::collections::HashMap::default();

        $(
            map.insert(stringify!($name).into(), $value.to_string());
        )*

        $crate::text_assets::get_text(&$s, map)
    }};
    ($prefix:expr, $s:expr) => {{
        let s: &str = $s.as_ref();
        let s = compact_str::format_compact!("{}/{}", $prefix, s);
        $crate::text_assets::get_text(&s, std::collections::HashMap::default())
    }};
    ($prefix:expr, $s:expr; $($name:ident = $value:expr),*) => {{
        let s: &str = $s.as_ref();
        let s = compact_str::format_compact!("{}/{}", $prefix, s);
        let mut map = std::collections::HashMap::default();

        $(
            map.insert(stringify!($name).into(), $value.to_string());
        )*

        $crate::text_assets::get_text(&s, map)
    }};
}

static LANG: AtomicCell<Lang> = AtomicCell::new(Lang::English);
static TRANSLATION_TEXTS: LazyLock<RwLock<HashMap<Lang, TranslationText>>> =
    LazyLock::new(|| RwLock::new(HashMap::default()));

pub fn set_lang(lang: Lang) {
    LANG.store(lang);
}

pub fn get_lang() -> Lang {
    LANG.load()
}

#[derive(Clone, Debug, Default, Asset, TypePath)]
pub struct TranslationText(HashMap<String, String>);

#[derive(Debug, Resource, AssetCollection)]
pub struct TranslationTexts {
    #[asset(path = "text/en", collection)]
    en: Vec<UntypedHandle>,
    #[asset(path = "text/ja", collection)]
    ja: Vec<UntypedHandle>,
}

pub fn set_text_global(
    mut commands: Commands,
    translation_texts: Res<TranslationTexts>,
    assets: Res<Assets<TranslationText>>,
    assets_random_name_list: Res<Assets<RandomNameList>>,
) {
    let mut map = HashMap::default();
    let mut random_name_list_map = HashMap::default();

    let langs = &[
        (Lang::English, &translation_texts.en),
        (Lang::Japanese, &translation_texts.ja),
    ];

    for (lang, handles) in langs {
        let mut text_handles = Vec::new();
        for handle in handles.iter() {
            if let Ok(handle) = handle.clone().try_typed::<TranslationText>() {
                text_handles.push(handle);
            } else if let Ok(handle) = handle.clone().try_typed::<RandomNameList>() {
                random_name_list_map.insert(
                    *lang,
                    assets_random_name_list.get(&handle).cloned().unwrap(),
                );
            }
        }
        let translation_text = TranslationText(
            text_handles
                .iter()
                .map(|h| assets.get(h).cloned().unwrap())
                .flat_map(|t| t.0.into_iter())
                .collect(),
        );
        map.insert(*lang, translation_text);
    }

    let t = &mut TRANSLATION_TEXTS.write().unwrap();
    **t = map;

    commands.insert_resource(RandomNameListMap(random_name_list_map));
}

pub fn get_text<S: AsRef<str>>(s: S, map: HashMap<String, String>) -> String {
    let s = s.as_ref();
    if let Some(translation_text) = TRANSLATION_TEXTS.read().unwrap().get(&LANG.load())
        && let Some(text) = translation_text.0.get(s)
    {
        if map.is_empty() {
            return text.into();
        } else {
            return replace(text, map);
        }
    }
    s.into()
}

pub fn replace(s: &str, map: HashMap<String, String>) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{\$([a-zA-Z][a-zA-Z0-9_-]*)\}").unwrap());

    RE.replace_all(s, |caps: &Captures| {
        let name = caps.get(1).unwrap().as_str();
        map.get(name)
            .map(|s| s.as_str())
            .unwrap_or_else(|| "{}")
            .to_owned()
    })
    .into_owned()
}

struct TextLoader;

impl AssetLoader for TextLoader {
    type Asset = TranslationText;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut dest_map = HashMap::default();
        let map: toml::Table = toml::from_str(std::str::from_utf8(&bytes)?)?;
        process_map(&map, &mut dest_map, &mut Vec::new())?;

        Ok(TranslationText(dest_map))
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}

fn process_map(
    map: &toml::Table,
    dest_map: &mut HashMap<String, String>,
    stack: &mut Vec<String>,
) -> anyhow::Result<()> {
    for (k, v) in map {
        match v {
            toml::Value::String(s) => {
                let mut a = stack.iter().fold(String::new(), |mut a, b| {
                    if !a.is_empty() {
                        a.push('/');
                    }
                    a.push_str(b);
                    a
                });
                if !a.is_empty() {
                    a.push('/');
                }
                a.push_str(k);
                dest_map.insert(a, s.clone());
            }
            toml::Value::Table(map) => {
                stack.push(k.clone());
                process_map(map, dest_map, stack)?;
                stack.pop();
            }
            value => bail!("cannot use \"{}\" for text", value),
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Asset, TypePath)]
pub struct RandomNameList(pub Vec<String>);

#[derive(Clone, Debug, Default, Resource)]
pub struct RandomNameListMap(pub HashMap<Lang, RandomNameList>);

struct RandomNameListLoader;

impl AssetLoader for RandomNameListLoader {
    type Asset = RandomNameList;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut s = String::new();
        reader.read_to_string(&mut s).await?;

        Ok(RandomNameList(
            s.lines()
                .filter_map(|line| {
                    if !line.trim_start().is_empty() {
                        Some(line.to_owned())
                    } else {
                        None
                    }
                })
                .collect(),
        ))
    }

    fn extensions(&self) -> &[&str] {
        &["random_name_list"]
    }
}
