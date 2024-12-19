use anyhow::bail;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
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
            .register_asset_loader(TextLoader);
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
        let s = compact_str::format_compact!("{}/{}", $prefix, $s);
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
    #[asset(
        paths(
            "text/en/base.toml",
            "text/en/help.toml",
            "text/en/planets.toml",
            "text/en/animals.toml",
        ),
        collection(typed)
    )]
    en: Vec<Handle<TranslationText>>,
    #[asset(
        paths(
            "text/ja/base.toml",
            "text/ja/help.toml",
            "text/ja/planets.toml",
            "text/ja/animals.toml",
        ),
        collection(typed)
    )]
    ja: Vec<Handle<TranslationText>>,
}

pub fn set_text_global(
    translation_texts: Res<TranslationTexts>,
    assets: Res<Assets<TranslationText>>,
) {
    let mut map = HashMap::default();

    map.insert(Lang::English, merge_text(&translation_texts.en, &assets));
    map.insert(Lang::Japanese, merge_text(&translation_texts.ja, &assets));

    let t = &mut TRANSLATION_TEXTS.write().unwrap();
    **t = map;
}

fn merge_text(t: &[Handle<TranslationText>], assets: &Assets<TranslationText>) -> TranslationText {
    TranslationText(
        t.iter()
            .map(|h| assets.get(h).cloned().unwrap())
            .flat_map(|t| t.0.into_iter())
            .collect(),
    )
}

pub fn get_text<S: AsRef<str>>(s: S, map: HashMap<String, String>) -> String {
    let s = s.as_ref();
    if let Some(translation_text) = TRANSLATION_TEXTS.read().unwrap().get(&LANG.load()) {
        if let Some(text) = translation_text.0.get(s) {
            if map.is_empty() {
                return text.into();
            } else {
                return replace(text, map);
            }
        }
    }
    s.into()
}

pub fn replace(s: &str, map: HashMap<String, String>) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{\$([a-zA-Z][a-zA-Z0-9_-]*)\}").unwrap());

    RE.replace_all(s, |caps: &Captures| {
        let name = caps.get(1).unwrap().as_str();
        map[name].to_string()
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
