use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_common_assets::ron::RonAssetPlugin;
use crossbeam::atomic::AtomicCell;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::{assets::TranslationTexts, planet::ResourceKind, GameState};

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, EnumIter, EnumString, AsRefStr,
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
        $crate::text::get_text($s, std::collections::HashMap::default())
    };
    ($s:expr; $($name:ident = $value:expr),*) => {{
        let mut map = std::collections::HashMap::default();

        $(
            map.insert(stringify!($name).into(), $value.to_string());
        )*

        $crate::text::get_text($s, map)
    }};
}

static LANG: AtomicCell<Lang> = AtomicCell::new(Lang::English);
static TRANSLATION_TEXTS: Lazy<RwLock<HashMap<Lang, TranslationText>>> =
    Lazy::new(|| RwLock::new(HashMap::default()));

pub fn set_lang(lang: Lang) {
    LANG.store(lang);
}

pub fn get_lang() -> Lang {
    LANG.load()
}

#[derive(Clone, Copy, Debug)]
pub struct TextPlugin;

#[derive(Clone, Debug, Default, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "c5967cb0-5b5a-433e-b659-8a96ff47422f"]
pub struct TranslationText(HashMap<String, String>);

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<TranslationText>::new(&["text.ron"]))
            .add_system(set_text.in_schedule(OnExit(GameState::AssetLoading)));
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
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\$([a-zA-Z][a-zA-Z0-9_-]*)\}").unwrap());

    RE.replace_all(s, |caps: &Captures| {
        let name = caps.get(1).unwrap().as_str();
        map[name].to_string()
    })
    .into_owned()
}

pub struct WithUnitDisplay {
    kind: ResourceKind,
    value: f32,
}

pub trait Unit {
    fn display_with_value(&self, value: f32) -> WithUnitDisplay;
}

impl Unit for ResourceKind {
    fn display_with_value(&self, value: f32) -> WithUnitDisplay {
        WithUnitDisplay { kind: *self, value }
    }
}

impl std::fmt::Display for WithUnitDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (unit0, unit1) = match self.kind {
            ResourceKind::Energy => ("PJ", "EJ"),
            _ => ("Mt", "Gt"),
        };

        if self.value < 1.0 {
            write!(f, "{}{}", self.value, unit0)
        } else if self.value < 100000.0 {
            write!(f, "{:.0}{}", self.value, unit0)
        } else {
            write!(f, "{:.0}{}", self.value / 1000.0, unit1)
        }
    }
}
