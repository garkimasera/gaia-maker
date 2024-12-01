use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator};

use super::WindowsOpenState;
use crate::planet::{BuildingAttrs, BuildingEffect, Params, SpaceBuildingKind, StructureKind};
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
}

impl AsRef<str> for HelpItem {
    fn as_ref(&self) -> &str {
        match self {
            HelpItem::Basics(basic_items) => basic_items.as_ref(),
            HelpItem::Structures(structure_kind) => structure_kind.as_ref(),
            HelpItem::SpaceBuildings(space_building_kind) => space_building_kind.as_ref(),
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

    map
});

pub fn help_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
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
                    ui.separator();
                    current_item.ui(ui, &params);
                });
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

impl HelpItem {
    pub fn ui(&self, ui: &mut egui::Ui, params: &Params) {
        if let Some(building_attrs) = match self {
            HelpItem::Structures(kind) => Some(&params.structures[kind].building),
            HelpItem::SpaceBuildings(kind) => Some(&params.space_buildings[kind]),
            _ => None,
        } {
            ui_building_attr(ui, building_attrs);
            ui.separator();
        }
        ui.label(t!(format!("help/{}", self.as_ref())));
    }
}

fn ui_building_attr(ui: &mut egui::Ui, attrs: &BuildingAttrs) {
    // Cost
    if attrs.cost > 0.0 {
        ui.label(egui::RichText::new(t!("cost")).strong());
        let s = format!(
            "{}: {}",
            t!("material"),
            WithUnitDisplay::Material(attrs.cost),
        );
        ui.label(s);
    }
    // Upkeep
    if attrs.energy < 0.0 {
        ui.label(egui::RichText::new(t!("upkeep")).strong());
        let s = format!(
            "{}: {}",
            t!("energy"),
            WithUnitDisplay::Energy(-attrs.energy),
        );
        ui.label(s);
    }
    // Produce
    let s = if attrs.energy > 0.0 {
        Some(format!(
            "{}: {}",
            t!("energy"),
            WithUnitDisplay::Energy(attrs.energy),
        ))
    } else if let Some(BuildingEffect::ProduceMaterial { mass }) = &attrs.effect {
        Some(format!(
            "{}: {}",
            t!("material"),
            WithUnitDisplay::Material(*mass),
        ))
    } else {
        None
    };
    if let Some(s) = s {
        ui.label(egui::RichText::new(t!("produce")).strong());
        ui.label(s);
    };
}
