use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator};

use super::{label_with_icon, EguiTextures, WindowsOpenState};
use crate::planet::{
    BuildingAttrs, BuildingEffect, Cost, Params, SpaceBuildingKind, StructureKind, TileEventKind,
};
use crate::screen::OccupiedScreenSpace;
use crate::text::WithUnitDisplay;

use std::collections::BTreeMap;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDiscriminants)]
#[strum_discriminants(name(ItemGroup))]
#[strum_discriminants(derive(PartialOrd, Ord, Hash, EnumIter, AsRefStr))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum HelpItem {
    Basics(BasicsItem),
    Structures(StructureKind),
    SpaceBuildings(SpaceBuildingKind),
    TileEvent(TileEventKind),
}

impl AsRef<str> for HelpItem {
    fn as_ref(&self) -> &str {
        match self {
            HelpItem::Basics(basic_items) => basic_items.as_ref(),
            HelpItem::Structures(structure_kind) => structure_kind.as_ref(),
            HelpItem::SpaceBuildings(space_building_kind) => space_building_kind.as_ref(),
            HelpItem::TileEvent(tile_event_kind) => tile_event_kind.as_ref(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum BasicsItem {
    #[default]
    Concept,
    Administrator,
    Terraforming,
}

#[allow(clippy::derivable_impls)]
impl Default for ItemGroup {
    fn default() -> Self {
        ItemGroup::Basics
    }
}

impl Default for HelpItem {
    fn default() -> Self {
        HelpItem::Basics(BasicsItem::Concept)
    }
}

impl From<ItemGroup> for HelpItem {
    fn from(group: ItemGroup) -> HelpItem {
        ITEM_LIST[&group][0]
    }
}

static ITEM_LIST: LazyLock<BTreeMap<ItemGroup, Vec<HelpItem>>> = LazyLock::new(|| {
    let mut map = BTreeMap::new();

    map.insert(
        ItemGroup::Basics,
        BasicsItem::iter().map(HelpItem::Basics).collect(),
    );
    map.insert(
        ItemGroup::Structures,
        StructureKind::iter()
            .filter_map(|structure_kind| {
                if matches!(structure_kind, StructureKind::Settlement) {
                    None
                } else {
                    Some(HelpItem::Structures(structure_kind))
                }
            })
            .collect(),
    );
    map.insert(
        ItemGroup::SpaceBuildings,
        SpaceBuildingKind::iter()
            .map(HelpItem::SpaceBuildings)
            .collect(),
    );
    map.insert(
        ItemGroup::TileEvent,
        TileEventKind::iter().map(HelpItem::TileEvent).collect(),
    );

    map
});

pub fn help_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    textures: Res<EguiTextures>,
    params: Res<Params>,
    mut current_item: Local<HelpItem>,
) {
    if !wos.help {
        return;
    }
    let rect = egui::Window::new(t!("help"))
        .open(&mut wos.help)
        .vscroll(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .show(ui, |ui| {
                        ui.set_min_width(150.0);
                        ui.set_min_height(300.0);
                        ui.vertical(|ui| {
                            for item_group in ItemGroup::iter() {
                                ui.collapsing(t!(item_group), |ui| {
                                    for item in &ITEM_LIST[&item_group] {
                                        ui.selectable_value(&mut *current_item, *item, t!(item));
                                    }
                                });
                            }
                        });
                    });
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_min_width(300.0);
                    ui.set_min_height(300.0);
                    ui.heading(t!(*current_item));
                    ui.separator();
                    current_item.ui(ui, &textures, &params);
                });
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

impl HelpItem {
    pub fn ui(&self, ui: &mut egui::Ui, textures: &EguiTextures, params: &Params) {
        if let Some(building_attrs) = match self {
            HelpItem::Structures(kind) => Some(&params.structures[kind].building),
            HelpItem::SpaceBuildings(kind) => Some(&params.space_buildings[kind]),
            _ => None,
        } {
            ui_building_attr(ui, textures, building_attrs);
            ui.separator();
        } else if let HelpItem::TileEvent(kind) = self {
            if let Some(cost) = &params.event.tile_event_costs.get(kind) {
                let (icon, s) = match **cost {
                    Cost::Material(value) => (
                        "ui/icon-material",
                        WithUnitDisplay::Material(value).to_string(),
                    ),
                    Cost::GenePoint(value) => (
                        "ui/icon-gene",
                        WithUnitDisplay::GenePoint(value).to_string(),
                    ),
                    _ => todo!(),
                };
                ui.label(egui::RichText::new(t!("cost")).strong());
                label_with_icon(ui, textures, icon, s);
                ui.separator();
            }
        }
        ui.label(t!("help", self));
    }
}

fn ui_building_attr(ui: &mut egui::Ui, textures: &EguiTextures, attrs: &BuildingAttrs) {
    // Cost
    if attrs.cost > 0.0 {
        ui.label(egui::RichText::new(t!("cost")).strong());
        label_with_icon(
            ui,
            textures,
            "ui/icon-material",
            WithUnitDisplay::Material(attrs.cost).to_string(),
        );
    }
    // Upkeep
    if attrs.energy < 0.0 {
        ui.label(egui::RichText::new(t!("upkeep")).strong());
        label_with_icon(
            ui,
            textures,
            "ui/icon-energy",
            WithUnitDisplay::Energy(-attrs.energy).to_string(),
        );
    }
    // Produce
    if attrs.energy > 0.0 {
        ui.label(egui::RichText::new(t!("produce")).strong());
        label_with_icon(
            ui,
            textures,
            "ui/icon-energy",
            WithUnitDisplay::Energy(attrs.energy).to_string(),
        );
    } else if let Some(BuildingEffect::ProduceMaterial { mass }) = &attrs.effect {
        ui.label(egui::RichText::new(t!("produce")).strong());
        label_with_icon(
            ui,
            textures,
            "ui/icon-material",
            WithUnitDisplay::Material(*mass).to_string(),
        );
    }
}
