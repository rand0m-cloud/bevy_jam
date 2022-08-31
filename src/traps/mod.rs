use crate::prelude::*;

pub mod alarm;
pub mod prox_bomb;

pub struct TrapsPlugin;
impl Plugin for TrapsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(prox_bomb::ProximityBombPlugin)
            .add_plugin(alarm::AlarmPlugin);
    }
}
