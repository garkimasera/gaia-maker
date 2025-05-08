use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator};

use super::{UiTextures, WindowsOpenState, misc::label_with_icon};
use crate::planet::{
    Biome, BuildingAttrs, BuildingEffect, CivilizationAge, Cost, EnergySource, GasKind, Params,
    SpaceBuildingKind, StructureKind, TileEventKind,
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
    Basics(&'static str),
    Facilities(StructureKind),
    SpaceBuildings(SpaceBuildingKind),
    TileEvents(TileEventKind),
    Biomes(Biome),
    Atmosphere(GasKind),
    CivilizationAges(CivilizationAge),
    EnergySources(EnergySource),
    Glossary(&'static str),
}

impl AsRef<str> for HelpItem {
    fn as_ref(&self) -> &str {
        match self {
            HelpItem::Basics(basic_items) => basic_items,
            HelpItem::Facilities(structure_kind) => structure_kind.as_ref(),
            HelpItem::SpaceBuildings(space_building_kind) => space_building_kind.as_ref(),
            HelpItem::TileEvents(tile_event_kind) => tile_event_kind.as_ref(),
            HelpItem::Biomes(biome) => biome.as_ref(),
            HelpItem::Atmosphere(gas_kind) => gas_kind.as_ref(),
            HelpItem::CivilizationAges(age) => match age {
                CivilizationAge::Stone => "age/stone",
                CivilizationAge::Bronze => "age/bronze",
                CivilizationAge::Iron => "age/iron",
                CivilizationAge::Industrial => "age/industrial",
                CivilizationAge::Atomic => "age/atomic",
                CivilizationAge::EarlySpace => "age/early-space",
            },
            HelpItem::EnergySources(energy_source) => match energy_source {
                EnergySource::Biomass => "energy_source/biomass",
                EnergySource::SolarWind => "energy_source/solar-wind",
                EnergySource::HydroGeothermal => "energy_source/hydro-geothermal",
                EnergySource::FossilFuel => "energy_source/fossil-fuel",
                EnergySource::Nuclear => "energy_source/nuclear",
                EnergySource::Gift => "energy_source/gift",
            },
            HelpItem::Glossary(word) => word,
        }
    }
}

const BASIC_ITEMS: &[&str] = &["concept", "player", "terraforming"];
const GLOSSARY_ITEMS: &[&str] = &["biomass", "civilization", "fertility", "solar-constant"];

#[allow(clippy::derivable_impls)]
impl Default for ItemGroup {
    fn default() -> Self {
        ItemGroup::Basics
    }
}

impl Default for HelpItem {
    fn default() -> Self {
        HelpItem::Basics("concept")
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
        BASIC_ITEMS.iter().map(|item| HelpItem::Basics(item)).collect(),
    );
    map.insert(
        ItemGroup::Facilities,
        StructureKind::iter()
            .filter_map(|structure_kind| {
                if matches!(structure_kind, StructureKind::Settlement) {
                    None
                } else {
                    Some(HelpItem::Facilities(structure_kind))
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
        ItemGroup::TileEvents,
        TileEventKind::iter().map(HelpItem::TileEvents).collect(),
    );
    map.insert(
        ItemGroup::Biomes,
        Biome::iter().map(HelpItem::Biomes).collect(),
    );
    map.insert(
        ItemGroup::Atmosphere,
        GasKind::iter().map(HelpItem::Atmosphere).collect(),
    );
    map.insert(
        ItemGroup::CivilizationAges,
        CivilizationAge::iter()
            .map(HelpItem::CivilizationAges)
            .collect(),
    );
    map.insert(
        ItemGroup::EnergySources,
        EnergySource::iter().map(HelpItem::EnergySources).collect(),
    );
    map.insert(
        ItemGroup::Glossary,
        GLOSSARY_ITEMS
            .iter()
            .map(|item| HelpItem::Glossary(item))
            .collect(),
    );

    map
});

pub fn help_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    textures: Res<UiTextures>,
    params: Res<Params>,
    mut current_item: Local<HelpItem>,
) {
    if !wos.help {
        return;
    }
    let ctx = egui_ctxs.ctx_mut();
    let rect = egui::Window::new(t!("help"))
        .constrain_to(super::misc::constrain_to_rect(ctx, &occupied_screen_space))
        .open(&mut wos.help)
        .vscroll(false)
        .show(ctx, |ui| {
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
    pub fn ui(&self, ui: &mut egui::Ui, textures: &UiTextures, params: &Params) {
        if let Some(building_attrs) = match self {
            HelpItem::Facilities(kind) => Some(&params.structures[kind].building),
            HelpItem::SpaceBuildings(kind) => Some(&params.space_buildings[kind]),
            _ => None,
        } {
            ui_building_attr(ui, textures, building_attrs);
            ui.separator();
        } else if let HelpItem::TileEvents(kind) = self {
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

fn ui_building_attr(ui: &mut egui::Ui, textures: &UiTextures, attrs: &BuildingAttrs) {
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
    if attrs.power < 0.0 {
        ui.label(egui::RichText::new(t!("upkeep")).strong());
        label_with_icon(
            ui,
            textures,
            "ui/icon-power",
            WithUnitDisplay::Power(-attrs.power).to_string(),
        );
    }
    // Produce
    if attrs.power > 0.0 {
        ui.label(egui::RichText::new(t!("produce")).strong());
        label_with_icon(
            ui,
            textures,
            "ui/icon-power",
            WithUnitDisplay::Power(attrs.power).to_string(),
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
