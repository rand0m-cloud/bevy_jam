use bevy_godot::prelude::*;

mod player;
mod zombies;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_plugin(player::PlayerPlugin)
        .add_plugin(zombies::ZombiesPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Component)]
pub struct Hp(f32);
