use crate::prelude::*;

pub mod ammo;
pub mod game_over;
pub mod item_bar;
pub mod score;
pub mod shelter;
pub mod stamina;
pub mod text_log;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(game_over::GameOverUiPlugin)
            .add_plugin(shelter::ShelterUiPlugin)
            .add_plugin(item_bar::ItemBarUiPlugin)
            .add_plugin(score::ScoreUiPlugin)
            .add_plugin(ammo::AmmoUiPlugin)
            .add_plugin(stamina::StaminaUiPlugin)
            .add_plugin(text_log::ItemLogPlugin);
    }
}
