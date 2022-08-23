use bevy_godot::prelude::*;

mod game_over;
mod shelter;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(game_over::GameOverUiPlugin)
            .add_plugin(shelter::ShelterUiPlugin);
    }
}
