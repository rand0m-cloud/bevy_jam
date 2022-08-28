use bevy::log::*;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::{bevy_prelude::Mut, *};
use rand::prelude::SliceRandom;
use std::collections::HashMap;

#[derive(Debug, AssetCollection)]
pub struct CraftingAssets {
    #[asset(path = "art/drone.tres")]
    proximity_bomb: Handle<GodotResource>,
    #[asset(path = "art/alarm_trap.tres")]
    alarm: Handle<GodotResource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, PartialOrd, Ord)]
pub enum Part {
    Battery,
    Electronics,
    Buzzer,
    Explosive,
    Motor,
}

impl Part {
    pub const ALL: &'static [Part] = &[
        Self::Battery,
        Self::Electronics,
        Self::Buzzer,
        Self::Explosive,
        //Self::Motor,
    ];

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        *Self::ALL.choose(&mut rng).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, PartialOrd, Ord)]
pub enum Item {
    Alarm,
    ProximityBomb,
    Drone,
}

impl Item {
    pub fn from_str(string: &str) -> Option<Self> {
        Some(match string {
            "Alarm" => Self::Alarm,
            "ProximityBomb" => Self::ProximityBomb,
            "Drone" => Self::Drone,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alarm => "Alarm",
            Self::ProximityBomb => "Proximity Bomb",
            Self::Drone => "Drone",
        }
    }

    pub fn as_texture_handle<'a>(&self, assets: &'a CraftingAssets) -> &'a Handle<GodotResource> {
        match self {
            Self::ProximityBomb => &assets.proximity_bomb,
            Self::Alarm => &assets.alarm,
            Self::Drone => todo!("missing drone art"),
        }
    }

    pub fn scene_path(&self) -> &'static str {
        match self {
            Self::ProximityBomb => "res://traps/ProximityBomb.tscn",
            Self::Alarm => "res://traps/Alarm.tscn",
            _ => panic!("do not have a scene path for {:?}", self),
        }
    }

    pub fn ingredients(&self) -> Vec<Part> {
        use Part::*;

        match self {
            Self::Alarm => {
                vec![Electronics, Battery, Buzzer]
            }
            Self::ProximityBomb => {
                vec![Electronics, Battery, Explosive]
            }
            Self::Drone => {
                vec![Electronics, Battery, Motor]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inventory {
    parts: HashMap<Part, u32>,
    items: HashMap<Item, u32>,
}

impl Inventory {
    pub fn can_craft(&self, item: Item) -> bool {
        let mut required = HashMap::new();
        for ingredient in item.ingredients() {
            *required.entry(ingredient).or_default() += 1;
        }

        for (part, count) in required {
            if self.parts.get(&part).copied().unwrap_or_default() < count {
                return false;
            }
        }

        true
    }

    pub fn craft(&mut self, item: Item) {
        if !self.can_craft(item) {
            return;
        }

        let mut required = HashMap::<Part, u32>::new();
        for ingredient in item.ingredients() {
            *required.entry(ingredient).or_default() += 1;
        }

        for (part, count) in required {
            *self.parts.get_mut(&part).unwrap() -= count;
        }

        info!("player crafted: {:?}", item);
        *self.items.entry(item).or_default() += 1;
    }

    pub fn add_part(&mut self, part: Part) {
        info!("giving player part: {:?}", part);
        *self.parts.entry(part).or_default() += 1;
    }

    pub fn add_parts(&mut self, parts: &[Part]) {
        for part in parts {
            self.add_part(*part);
        }
    }

    pub fn get_items(&self) -> &HashMap<Item, u32> {
        &self.items
    }

    pub fn use_item(&mut self, item: &Item) {
        if let Some(count) = self.items.get_mut(item) {
            *count -= 1;
        } else {
            warn!("tried to use item: {:?} but did not have any", item);
        }
    }

    pub fn get_parts(&self) -> &HashMap<Part, u32> {
        &self.parts
    }
}
