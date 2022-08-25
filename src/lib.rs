#![allow(clippy::type_complexity)]

use bevy_godot::prelude::*;
use iyes_loopless::prelude::*;

mod airdrops;
mod crafting;
mod player;
mod traps;
mod ui;
mod zombies;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_loopless_state(GameState::Playing)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(zombies::ZombiesPlugin)
        .add_plugin(airdrops::AirDropsPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(traps::TrapsPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Component)]
pub struct Hp(f32);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Playing,
    Sheltered,
    GameOver,
}
