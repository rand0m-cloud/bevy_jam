#![allow(clippy::type_complexity)]

use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Instant;

pub mod airdrops;
mod crafting;
mod player;
mod traps;
mod ui;
mod zombies;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_loopless_state(GameState::Loading)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .with_collection::<zombies::ZombieAssets>()
                .with_collection::<crafting::CraftingAssets>(),
        )
        .insert_resource(Score(0))
        .insert_resource(SelectedItemSlot(Some(0)))
        .add_exit_system(GameState::Loading, set_round_start)
        .add_exit_system(GameState::GameOver, set_round_start)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(zombies::ZombiesPlugin)
        .add_plugin(airdrops::AirDropsPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(traps::TrapsPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Component)]
pub struct Hp(f32);

pub struct Score(pub u64);

pub struct SelectedItemSlot(Option<u16>);

#[derive(Debug)]
pub struct RoundStart(pub Instant);

fn set_round_start(mut commands: Commands) {
    commands.insert_resource(RoundStart(Instant::now()));
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Loading,
    Playing,
    Sheltered,
    GameOver,
}
