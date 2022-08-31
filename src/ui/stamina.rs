use crate::player::prelude::*;
use crate::prelude::*;
use bevy_godot::prelude::godot_prelude::ProgressBar;

pub struct StaminaUiPlugin;
impl Plugin for StaminaUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_stamina_ui).add_system(
            update_stamina_ui
                .as_visual_system()
                .run_not_in_state(GameState::Loading),
        );
    }
}

#[derive(Component, Debug)]
pub struct StaminaUiLabel;

fn label_stamina_ui(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let score_ui_ent = entities
        .iter()
        .find_entity_by_name("StaminaProgressBar")
        .unwrap();

    commands.entity(score_ui_ent).insert(StaminaUiLabel);
}

fn update_stamina_ui(
    mut stamina_ui: Query<&mut ErasedGodotRef, With<StaminaUiLabel>>,
    player: Query<&PlayerStamina, Changed<PlayerStamina>>,
) {
    if let Ok(stamina) = player.get_single() {
        let mut stamina_ui = stamina_ui.single_mut();
        stamina_ui.get::<ProgressBar>().set_value(stamina.0 as f64);
    }
}
