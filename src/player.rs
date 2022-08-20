use bevy_godot::prelude::godot_prelude::Vector2;
use bevy_godot::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_player).add_system(move_player);
    }
}

#[derive(Debug, Component)]
pub struct Player;

fn label_player(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let player_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "Player").then_some(ent))
        .unwrap();

    commands.entity(player_ent).insert(Player);
}

fn move_player(mut player: Query<(&Player, &mut Transform2D)>, time: Res<Time>) {
    let (_, mut player_transform) = player.single_mut();
    let input = Input::godot_singleton();

    let move_forward = input.get_action_strength("move_forward", false);
    let move_backward = input.get_action_strength("move_backward", false);

    let rotate_left = input.get_action_strength("rotate_left", false);
    let rotate_right = input.get_action_strength("rotate_right", false);

    let move_input = move_backward - move_forward;
    let rotate_input = rotate_right - rotate_left;

    player_transform.origin =
        player_transform.xform(Vector2::new(0.0, move_input as f32) * 50.0 * time.delta_seconds());

    let rotation = player_transform.rotation();
    player_transform.set_rotation(rotate_input as f32 * 1.5 * time.delta_seconds() + rotation);
}
