use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use crossbeam::atomic::AtomicCell;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::{
    assets::TranslationTexts,
    planet::{Msg, MsgKind},
    GameState,
};

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumIter,
    EnumString,
    AsRefStr,
    Reflect,
)]
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
        $crate::text::get_text(&$s, std::collections::HashMap::default())
    };
    ($s:expr; $($name:ident = $value:expr),*) => {{
        let mut map = std::collections::HashMap::default();

        $(
            map.insert(stringify!($name).into(), $value.to_string());
        )*

        $crate::text::get_text(&$s, map)
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

#[derive(Clone, Copy, Debug)]
pub struct TextPlugin;

#[derive(Clone, Debug, Default, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct TranslationText(HashMap<String, String>);

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<TranslationText>::new(&["text.ron"]))
            .add_systems(OnExit(GameState::AssetLoading), set_text);
    }
}

fn set_text(translation_texts: Res<TranslationTexts>, texts: Res<Assets<TranslationText>>) {
    let t = &mut TRANSLATION_TEXTS.write().unwrap();
    for lang in Lang::iter() {
        let Some(translation_text) = texts.get(&translation_texts.get(lang)) else {
            continue;
        };
        t.insert(lang, translation_text.clone());
    }
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WithUnitDisplay {
    Energy(f32),
    Material(f32),
    GenePoint(f32),
}

impl std::fmt::Display for WithUnitDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            WithUnitDisplay::Energy(value) => {
                write!(f, "{}TW", value)
            }
            WithUnitDisplay::Material(value) => {
                if value < 1.0 {
                    write!(f, "{}Mt", value)
                } else if value < 100000.0 {
                    write!(f, "{:.0}Mt", value)
                } else {
                    write!(f, "{:.0}Gt", value / 1000.0)
                }
            }
            WithUnitDisplay::GenePoint(value) => {
                write!(f, "{:.0}", value)
            }
        }
    }
}

impl Msg {
    pub fn text(&self) -> (MsgStyle, String) {
        use MsgStyle::*;
        match &self.kind {
            MsgKind::WarnHighTemp => (Warn, t!("msg/warn-high-temp")),
            MsgKind::WarnLowTemp => (Warn, t!("msg/warn-low-temp")),
            MsgKind::EventStart => (Notice, t!("event/start")),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MsgStyle {
    Notice,
    Warn,
}

impl MsgStyle {
    pub fn icon(&self) -> &str {
        match self {
            MsgStyle::Notice => "ℹ",
            MsgStyle::Warn => "⚠",
        }
    }
}

pub fn split_to_head_body(s: &str) -> (&str, Option<&str>) {
    if let Some((head, body)) = s.split_once('\n') {
        (head, Some(body))
    } else {
        (s, None)
    }
}
