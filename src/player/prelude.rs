use crate::player::weapon::{Bullet, WeaponAssets};
use crate::prelude::*;

pub use crate::player::movement::PlayerTrapTarget;

#[derive(Debug, Component, Default)]
pub struct Player;

impl Player {
    pub fn label_system(mut commands: Commands, entities: Query<(&Name, Entity)>) {
        let player = entities.iter().find_entity_by_name("Player").unwrap();
        commands
            .entity(player)
            .insert_bundle(PlayerBundle::default());
    }

    pub fn restart_system(mut commands: Commands, player: Query<Entity, With<Player>>) {
        commands
            .entity(player.single())
            .insert_bundle(PlayerBundle::default());
    }
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    player_activity: PlayerActivity,
    player_stamina: PlayerStamina,
    player_inventory: PlayerInventory,
    player_weapon: PlayerWeapon,
}

#[derive(Debug, PartialEq, Eq, Hash, Component, Default, Copy, Clone)]
pub enum PlayerActivity {
    #[default]
    Standing,
    Walking,
    Running,
}

impl PlayerActivity {
    pub fn movement_speed(&self) -> f32 {
        match self {
            Self::Standing => 0.0,
            Self::Walking => 70.0,
            Self::Running => 165.0,
        }
    }
}

#[derive(Debug, Component, Default, Copy, Clone)]
pub struct PlayerInteractVolume;

impl PlayerInteractVolume {
    pub fn label_system(mut commands: Commands, mut entities: Query<(&Name, Entity)>) {
        let volume = entities
            .iter_mut()
            .find_entity_by_name("PlayerInteractVolume")
            .unwrap();
        commands.entity(volume).insert(PlayerInteractVolume);
    }
}

#[derive(Debug, Component, Copy, Clone)]
pub struct PlayerStamina(pub f64);

impl Default for PlayerStamina {
    fn default() -> Self {
        Self(1.0)
    }
}

impl PlayerStamina {
    pub fn tick(&mut self, activity: &PlayerActivity, delta: Duration) {
        let fatigue = match activity {
            PlayerActivity::Running => -0.15,
            _ => 0.3,
        };
        self.0 = f64::min(1.0, fatigue * delta.as_secs_f64() + self.0);
    }

    pub fn can_run(&self) -> bool {
        self.0 > 0.01
    }
}
#[derive(Debug, Component, Clone)]
pub struct PlayerWeapon {
    pub targets: Vec<Entity>,
    pub reload_timer: Timer,
}

impl Default for PlayerWeapon {
    fn default() -> Self {
        Self {
            targets: vec![],
            reload_timer: Timer::from_seconds(0.2, false),
        }
    }
}

impl PlayerWeapon {
    pub fn fire(
        &mut self,
        commands: &mut Commands,
        assets: &WeaponAssets,
        transform: &GodotTransform2D,
    ) {
        commands
            .spawn()
            .insert(GodotScene::from_handle(&assets.bullet))
            .insert(Bullet)
            .insert(Transform2D(*transform));
    }
}

#[derive(Debug, Component, Clone, PartialEq, Eq)]
pub struct PlayerInventory(pub Inventory);

impl Default for PlayerInventory {
    fn default() -> Self {
        // set up initial inventory with parts for a bomb and an alarm
        let mut inventory = Inventory::default();
        inventory.add_parts(&Item::ProximityBomb.ingredients());
        inventory.add_parts(&Item::Alarm.ingredients());
        inventory.add_ammo(15);

        Self(inventory)
    }
}

impl std::ops::Deref for PlayerInventory {
    type Target = Inventory;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PlayerInventory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
