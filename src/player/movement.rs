use crate::player::prelude::*;
use crate::prelude::*;
use bevy_godot::prelude::godot_prelude::{Physics2DServer, RigidBody2D};

pub struct PlayerMovementPlugin;
impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(PlayerTrapTarget::label_system)
            .add_system(move_player.as_physics_system())
            .add_system(
                update_trap_target
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(update_activity.as_physics_system())
            .add_system(update_stamina.as_physics_system())
            .add_enter_system(GameState::GameOver, game_over);
    }
}

#[derive(Debug, Component)]
pub struct PlayerTrapTarget;

impl PlayerTrapTarget {
    fn label_system(mut commands: Commands, entities: Query<(&Name, Entity)>) {
        let move_target = entities.iter().find_entity_by_name("TrapTarget").unwrap();

        commands.entity(move_target).insert(PlayerTrapTarget);
    }
}

fn move_player(
    mut player: Query<(&mut ErasedGodotRef, &PlayerActivity)>,
    state: Res<CurrentState<GameState>>,
    _scene_tree_ref: SceneTreeRef,
) {
    let (mut player, player_activity) = player.single_mut();

    let physics_server = unsafe { Physics2DServer::godot_singleton() };
    let body = unsafe {
        physics_server
            .body_get_direct_state(player.get::<RigidBody2D>().get_rid())
            .unwrap()
            .assume_safe()
    };

    if matches!(state.0, GameState::Sheltered | GameState::GameOver) {
        body.set_linear_velocity(Vector2::ZERO);
        return;
    }

    let input = Input::godot_singleton();
    let input_dir = input.get_vector("move_left", "move_right", "move_up", "move_down", -1.0);

    let velocity = input_dir * player_activity.movement_speed();

    body.set_linear_velocity(velocity);
    body.set_angular_velocity(1.0);
}

fn update_trap_target(
    mut move_target: Query<&mut Transform2D, (With<PlayerTrapTarget>, Without<Player>)>,
    player: Query<&Transform2D, With<Player>>,
    mut time: SystemDeltaTimer,
) {
    let mut transform = move_target.single_mut();
    let player_transform = player.single();
    let delta = time.delta_seconds();

    let input = Input::godot_singleton();
    let input_dir = input.get_vector(
        "move_trap_target_left",
        "move_trap_target_right",
        "move_trap_target_up",
        "move_trap_target_down",
        -1.0,
    );

    if input_dir.length() != 0.0 {
        let delta = input_dir * 500.0 * delta;
        transform.origin += delta;
    }

    let trap_target_dir = transform.origin - player_transform.origin;
    let max_trap_target_distance = 250.0;

    if trap_target_dir.length() > max_trap_target_distance {
        transform.origin =
            trap_target_dir.normalized() * max_trap_target_distance + player_transform.origin;
    }

    if input.is_action_just_pressed("reset_trap_target", false) {
        transform.origin = player_transform.origin;
    }
}

fn update_activity(mut player: Query<(&mut PlayerActivity, &PlayerStamina), With<Player>>) {
    let (mut player_activity, player_stamina) = player.single_mut();

    let input = Input::godot_singleton();
    let input_dir = input.get_vector("move_left", "move_right", "move_up", "move_down", -1.0);

    let run_pressed = input.is_action_just_pressed("sprint", false);
    let run_released = input.is_action_just_released("sprint", false);

    match *player_activity {
        _ if input_dir.length() == 0.0 => {
            *player_activity = PlayerActivity::Standing;
        }
        PlayerActivity::Standing => {
            *player_activity = PlayerActivity::Walking;
        }
        PlayerActivity::Walking if run_pressed && player_stamina.can_run() => {
            *player_activity = PlayerActivity::Running;
        }
        PlayerActivity::Running if run_released || !player_stamina.can_run() => {
            *player_activity = PlayerActivity::Walking;
        }
        _ => {}
    }
}

fn update_stamina(
    mut player: Query<(&mut PlayerStamina, &PlayerActivity)>,
    mut time: SystemDeltaTimer,
    state: Res<CurrentState<GameState>>,
) {
    let delta = time.delta();

    if state.0 != GameState::Playing {
        return;
    }

    let (mut stamina, activity) = player.single_mut();
    stamina.tick(activity, delta);
}

fn game_over() {}
