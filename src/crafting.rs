use crate::prelude::*;

#[derive(Debug, AssetCollection)]
pub struct CraftingAssets {
    #[asset(path = "art/bomb.tres")]
    proximity_bomb: Handle<GodotResource>,

    #[asset(path = "traps/ProximityBomb.tscn")]
    proximity_bomb_scene: Handle<GodotResource>,

    #[asset(path = "art/alarm_trap.tres")]
    alarm: Handle<GodotResource>,

    #[asset(path = "traps/Alarm.tscn")]
    alarm_scene: Handle<GodotResource>,
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
        *Self::ALL
            .choose_weighted(&mut rng, Self::loot_weight)
            .unwrap()
    }

    pub fn loot_weight(&self) -> u32 {
        use Part::*;

        match self {
            Battery | Electronics => 7,
            _ => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, PartialOrd, Ord)]
pub enum Item {
    ProximityBomb,
    Alarm,
    Drone,
}

impl FromStr for Item {
    type Err = ();
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(match string {
            "Alarm" => Self::Alarm,
            "ProximityBomb" => Self::ProximityBomb,
            "Drone" => Self::Drone,
            _ => return Err(()),
        })
    }
}

impl Item {
    pub const ALL: &'static [Self] = &[Self::ProximityBomb, Self::Alarm];

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

    pub fn as_scene_handle<'a>(&self, assets: &'a CraftingAssets) -> &'a Handle<GodotResource> {
        match self {
            Self::ProximityBomb => &assets.proximity_bomb_scene,
            Self::Alarm => &assets.alarm_scene,
            Self::Drone => todo!("missing drone scene"),
        }
    }

    pub fn ingredients(&self) -> Vec<Part> {
        use Part::*;

        #[derive(Default)]
        struct RecipeBuilder(Vec<Part>);

        impl RecipeBuilder {
            fn add_ingredients(
                &mut self,
                ingredients: impl IntoIterator<Item = Part>,
            ) -> &mut Self {
                self.0.extend(ingredients);
                self
            }

            fn finish(self) -> Vec<Part> {
                self.0
            }
        }

        let mut recipe = RecipeBuilder::default();

        match self {
            Self::Alarm => recipe.add_ingredients([Electronics, Battery, Buzzer]),
            Self::ProximityBomb => recipe.add_ingredients([Electronics, Battery, Explosive]),
            Self::Drone => recipe.add_ingredients([Electronics, Battery, Motor]),
        };
        recipe.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inventory {
    parts: HashMap<Part, u32>,
    items: HashMap<Item, u32>,
    ammo_count: u32,
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

    pub fn use_ammo(&mut self, count: u32) {
        self.ammo_count -= count;
    }

    pub fn add_ammo(&mut self, count: u32) {
        self.ammo_count += count;
    }

    pub fn ammo_count(&self) -> u32 {
        self.ammo_count
    }
}
