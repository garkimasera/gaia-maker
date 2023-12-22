use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use once_cell::sync::Lazy;
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoEnumIterator};

use super::{convert_rect, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::{
    BuildingAttrs, OrbitalBuildingKind, Params, StarSystemBuildingKind, StructureKind,
};
use crate::screen::OccupiedScreenSpace;
use crate::text::Unit;

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDiscriminants)]
#[strum_discriminants(name(ItemGroup))]
#[strum_discriminants(derive(PartialOrd, Ord, Hash, EnumIter, AsRefStr))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum HelpItem {
    Basics(BasicsItem),
    Structures(StructureKind),
    OrbitalBuildings(OrbitalBuildingKind),
    StarSystemBuildings(StarSystemBuildingKind),
}

impl AsRef<str> for HelpItem {
    fn as_ref(&self) -> &str {
        match self {
            HelpItem::Basics(basic_items) => basic_items.as_ref(),
            HelpItem::Structures(structure_kind) => structure_kind.as_ref(),
            HelpItem::OrbitalBuildings(orbital_building_kind) => orbital_building_kind.as_ref(),
            HelpItem::StarSystemBuildings(star_system_building_kind) => {
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

static ITEM_LIST: Lazy<BTreeMap<ItemGroup, Vec<HelpItem>>> = Lazy::new(|| {
    let mut map = BTreeMap::new();

    map.insert(
        ItemGroup::Basics,
        BasicsItem::iter().map(HelpItem::Basics).collect(),
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
                    Some(HelpItem::Structures(structure_kind))
                }
            })
            .collect(),
    );
    map.insert(
        ItemGroup::OrbitalBuildings,
        OrbitalBuildingKind::iter()
            .map(HelpItem::OrbitalBuildings)
            .collect(),
    );
    map.insert(
        ItemGroup::StarSystemBuildings,
        StarSystemBuildingKind::iter()
            .map(HelpItem::StarSystemBuildings)
            .collect(),
    );

    map
});

pub fn help_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
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
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.ui.scale_factor));
}

impl HelpItem {
    pub fn ui(&self, ui: &mut egui::Ui, params: &Params) {
        if let Some(building_attrs) = match self {
            HelpItem::Structures(kind) => Some(&params.structures[kind].building),
            HelpItem::OrbitalBuildings(kind) => Some(&params.orbital_buildings[kind]),
            HelpItem::StarSystemBuildings(kind) => Some(&params.star_system_buildings[kind]),
            _ => None,
        } {
            ui_building_attr(ui, building_attrs);
            ui.separator();
        }
        ui.label(t!(format!("help/{}", self.as_ref())));
    }
}

fn ui_building_attr(ui: &mut egui::Ui, attrs: &BuildingAttrs) {
    if !attrs.cost.is_empty() {
        ui.label(egui::RichText::new(t!("cost")).strong());
        let mut resources = attrs.cost.iter().collect::<Vec<_>>();
        resources.sort_by_key(|(resource, _)| *resource);
        let s = resources
            .into_iter()
            .map(|(resource, value)| {
                format!(
                    "{}: {}",
                    t!(resource.as_ref()),
                    resource.display_with_value(*value)
                )
            })
            .fold(String::new(), |mut s0, s1| {
                if !s0.is_empty() {
                    s0.push_str(", ");
                }
                s0.push_str(&s1);
                s0
            });
        ui.label(s);
    }
    if !attrs.upkeep.is_empty() {
        ui.label(egui::RichText::new(t!("upkeep")).strong());
        let mut resources = attrs.upkeep.iter().collect::<Vec<_>>();
        resources.sort_by_key(|(resource, _)| *resource);
        let s = resources
            .iter()
            .map(|(resource, value)| {
                format!(
                    "{}: {}",
                    t!(resource.as_ref()),
                    resource.display_with_value(**value)
                )
            })
            .fold(String::new(), |mut s0, s1| {
                if !s0.is_empty() {
                    s0.push_str(", ");
                }
                s0.push_str(&s1);
                s0
            });
        ui.label(s);
    }
    if !attrs.produce.is_empty() {
        ui.label(egui::RichText::new(t!("produce")).strong());
        let mut resources = attrs.produce.iter().collect::<Vec<_>>();
        resources.sort_by_key(|(resource, _)| *resource);
        let s = resources
            .iter()
            .map(|(resource, value)| {
                format!(
                    "{}: {}",
                    t!(resource.as_ref()),
                    resource.display_with_value(**value)
                )
            })
            .fold(String::new(), |mut s0, s1| {
                if !s0.is_empty() {
                    s0.push_str(", ");
                }
                s0.push_str(&s1);
                s0
            });
        ui.label(s);
    }
}
