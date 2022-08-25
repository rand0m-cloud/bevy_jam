use crate::player::Player;
use bevy_godot::prelude::{
    bevy_prelude::{Changed, With},
    *,
};

pub struct AmmoUiPlugin;
impl Plugin for AmmoUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_ammo_ui)
            .add_system(update_ammo_ui);
    }
}

#[derive(Component, Debug)]
pub struct AmmoLabel;

fn label_ammo_ui(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let score_ui_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AmmoLabel").then_some(ent))
        .unwrap();

    commands.entity(score_ui_ent).insert(AmmoLabel);
}

fn update_ammo_ui(
    mut ammo_ui: Query<&mut ErasedGodotRef, With<AmmoLabel>>,
    player: Query<&Player, Changed<Player>>,
) {
    if let Ok(player) = player.get_single() {
        let mut ammo_ui = ammo_ui.single_mut();
        ammo_ui
            .get::<Label>()
            .set_text(format!("Ammo Remaining: {}", player.ammo_count));
    }
}
