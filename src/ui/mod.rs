use bevy_godot::prelude::*;

mod ammo;
mod game_over;
mod item_bar;
mod score;
mod shelter;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(game_over::GameOverUiPlugin)
            .add_plugin(shelter::ShelterUiPlugin)
            .add_plugin(item_bar::ItemBarUiPlugin)
            .add_plugin(score::ScoreUiPlugin)
            .add_plugin(ammo::AmmoUiPlugin);
    }
}
