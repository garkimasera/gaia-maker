use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_common_assets::ron::RonAssetPlugin;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::planet::ResourceKind;

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

static TRANSLATION_TEXTS: Lazy<RwLock<HashMap<String, TranslationText>>> =
    Lazy::new(|| RwLock::new(HashMap::default()));
static LANG_CODE: Lazy<String> = Lazy::new(lang_code);

#[derive(Clone, Copy, Debug)]
pub struct TextPlugin;

#[derive(Clone, Debug, Default, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "c5967cb0-5b5a-433e-b659-8a96ff47422f"]
pub struct TranslationText(HashMap<String, String>);

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<TranslationText>::new(&["text.ron"]))
            .init_resource::<TextLoading>()
            .add_startup_system(load_text)
            .add_system(update_text);
    }
}

#[derive(Default, Resource)]
struct TextLoading(Vec<HandleUntyped>);

fn load_text(asset_server: Res<AssetServer>, mut text_loading: ResMut<TextLoading>) {
    text_loading
        .0
        .push(asset_server.load_untyped(&format!("texts/{}.text.ron", lang_code())));
}

fn update_text(
    mut command: Commands,
    asset_server: Res<AssetServer>,
    loading: Option<Res<TextLoading>>,
    texts: Res<Assets<TranslationText>>,
) {
    let loading = if let Some(loading) = loading {
        loading
    } else {
        return;
    };
    match asset_server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
        LoadState::Failed => {
            panic!();
        }
        LoadState::Loaded => (),
        _ => {
            return;
        }
    }

    let texts = texts
        .iter()
        .map(|(id, text)| {
            (
                asset_server
                    .get_handle_path(id)
                    .unwrap()
                    .path()
                    .display()
                    .to_string()
                    .strip_prefix("texts/")
                    .unwrap()
                    .strip_suffix(".text.ron")
                    .unwrap()
                    .to_string(),
                text.clone(),
            )
        })
        .collect::<HashMap<String, TranslationText>>();

    *TRANSLATION_TEXTS.write().unwrap() = texts;

    crate::msg::push_msg(
        crate::msg::MsgKind::Notice,
        t!("welcome_to"; app_name=crate::APP_NAME),
    );

    command.remove_resource::<TextLoading>();
}

fn lang_code() -> String {
    if let Ok(lang) = std::env::var("LANG") {
        if let Some(lang) = lang.split('_').next() {
            return lang.into();
        }
    }
    "en".into()
}

pub fn get_text(s: &str, map: HashMap<String, String>) -> String {
    if let Some(translation_text) = TRANSLATION_TEXTS.read().unwrap().get(&*LANG_CODE) {
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
