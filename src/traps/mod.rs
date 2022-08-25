use bevy_godot::prelude::*;

mod prox_bomb;

pub struct TrapsPlugin;
impl Plugin for TrapsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(prox_bomb::ProximityBombPlugin);
    }
}
