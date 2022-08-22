#![allow(clippy::type_complexity)]

use bevy_godot::prelude::*;

mod airdrops;
mod player;
mod zombies;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_plugin(player::PlayerPlugin)
        .add_plugin(zombies::ZombiesPlugin)
        .add_plugin(airdrops::AirDropsPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Component)]
pub struct Hp(f32);
