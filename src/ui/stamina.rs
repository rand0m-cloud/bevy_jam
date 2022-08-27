use crate::{
    player::{Player, Stamina},
    GameState,
};
use bevy_godot::prelude::{
    bevy_prelude::{Changed, With},
    *,
};
use iyes_loopless::prelude::*;

pub struct StaminaUiPlugin;
impl Plugin for StaminaUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_stamina_ui)
            .add_system(update_stamina_ui.run_not_in_state(GameState::Loading));
    }
}

#[derive(Component, Debug)]
pub struct StaminaUiLabel;

fn label_stamina_ui(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let score_ui_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "StaminaProgressBar").then_some(ent))
        .unwrap();

    commands.entity(score_ui_ent).insert(StaminaUiLabel);
}

fn update_stamina_ui(
    mut stamina_ui: Query<&mut ErasedGodotRef, With<StaminaUiLabel>>,
    player: Query<&Stamina, (Changed<Stamina>, With<Player>)>,
) {
    if let Ok(stamina) = player.get_single() {
        let mut stamina_ui = stamina_ui.single_mut();
        stamina_ui.get::<ProgressBar>().set_value(stamina.0 as f64);
    }
}
