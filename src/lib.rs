use bevy_godot::prelude::*;

mod player;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_plugin(player::PlayerPlugin);
}

bevy_godot_init!(init, build_app);
