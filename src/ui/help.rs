use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use once_cell::sync::Lazy;
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator};

use super::{convert_rect, WindowsOpenState};
use crate::planet::{OrbitalBuildingKind, StarSystemBuildingKind, StructureKind};
use crate::{conf::Conf, screen::OccupiedScreenSpace};

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDiscriminants)]
#[strum_discriminants(name(ItemGroup))]
#[strum_discriminants(derive(PartialOrd, Ord, Hash, EnumIter, AsRefStr))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Item {
    Basics(BasicsItem),
    Structures(StructureKind),
    OrbitalBuildings(OrbitalBuildingKind),
    StarSystemBuildings(StarSystemBuildingKind),
}

impl AsRef<str> for Item {
    fn as_ref(&self) -> &str {
        match self {
            Item::Basics(basic_items) => basic_items.as_ref(),
            Item::Structures(structure_kind) => structure_kind.as_ref(),
            Item::OrbitalBuildings(orbital_building_kind) => orbital_building_kind.as_ref(),
            Item::StarSystemBuildings(star_system_building_kind) => {
                star_system_building_kind.as_ref()
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum BasicsItem {
    #[default]
    Concept,
}

impl Default for ItemGroup {
    fn default() -> Self {
        ItemGroup::Basics
    }
}

impl Default for Item {
    fn default() -> Self {
        Item::Basics(BasicsItem::Concept)
    }
}

impl From<ItemGroup> for Item {
    fn from(group: ItemGroup) -> Item {
        ITEM_LIST[&group][0]
    }
}

static ITEM_LIST: Lazy<BTreeMap<ItemGroup, Vec<Item>>> = Lazy::new(|| {
    let mut map = BTreeMap::new();

    map.insert(
        ItemGroup::Basics,
        BasicsItem::iter().map(Item::Basics).collect(),
    );
    map.insert(
        ItemGroup::Structures,
        StructureKind::iter()
            .filter_map(|structure_kind| {
                if matches!(
                    structure_kind,
                    StructureKind::None | StructureKind::Occupied
                ) {
                    None
                } else {
                    Some(Item::Structures(structure_kind))
                }
            })
            .collect(),
    );
    map.insert(
        ItemGroup::OrbitalBuildings,
        OrbitalBuildingKind::iter()
            .map(Item::OrbitalBuildings)
            .collect(),
    );
    map.insert(
        ItemGroup::StarSystemBuildings,
        StarSystemBuildingKind::iter()
            .map(Item::StarSystemBuildings)
            .collect(),
    );

    map
});

pub fn help_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
    mut current_item: Local<Item>,
) {
    if !wos.help {
        return;
    }
    let rect = egui::Window::new(t!("help"))
        .open(&mut wos.help)
        .vscroll(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .show(ui, |ui| {
                        ui.set_min_width(150.0);
                        ui.set_min_height(300.0);
                        ui.vertical(|ui| {
                            for item_group in ItemGroup::iter() {
                                ui.collapsing(t!(item_group.as_ref()), |ui| {
                                    for item in &ITEM_LIST[&item_group] {
                                        ui.selectable_value(
                                            &mut *current_item,
                                            *item,
                                            t!(item.as_ref()),
                                        );
                                    }
                                });
                            }
                        });
                    });
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_min_width(300.0);
                    ui.set_min_height(300.0);
                    ui.heading(t!(current_item.as_ref()));
                    ui.label(t!(format!("help/{}", current_item.as_ref())));
                });
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}
