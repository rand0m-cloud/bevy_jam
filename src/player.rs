use crate::{
    crafting::{Inventory, Item},
    zombies::Zombie,
    GameState, Hp, SelectedItemSlot,
};
use bevy::log::*;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With, Without},
    godot_prelude::Vector2,
    *,
};
use iyes_loopless::prelude::*;

// TODO: Is there a way to set those in Godot and read them here? It would be nice to be able to experiment with constants on the fly.
const WALKING_SPEED: f32 = 40.0;
const RUNNING_SPEED: f32 = 100.0;
const TURNING_SPEED: f64 = 4.0;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_player)
            .add_startup_system(label_shot_audio)
            .add_startup_system(label_goal)
            .add_startup_system(label_target)
            .add_system(
                move_player
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(aim.as_physics_system().run_in_state(GameState::Playing))
            .add_system(
                set_goal
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(
                toggle_ducking
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(
                toggle_running
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(
                player_shoot
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(setup_bullet.as_physics_system())
            .add_system(damage_bullet)
            .add_system(place_trap.as_physics_system())
            .add_exit_system(GameState::GameOver, on_restart);
    }
}

#[derive(Debug, Component)]
pub struct Player {
    pub inventory: Inventory,
    pub ammo_count: u32,
}

impl Default for Player {
    fn default() -> Self {
        // setup initial inventory with parts for a bomb
        let mut inventory = Inventory::default();
        inventory.add_parts(&Item::ProximityBomb.ingredients());

        Player {
            inventory,
            ammo_count: 15,
        }
    }
}

impl Player {
    fn reset(&mut self) {
        *self = Self::default();
    }
}
#[derive(Debug, Component)]
pub struct PlayerInteractVolume;

// A goal represents a point where the player is going
#[derive(Debug, Component)]
struct Goal;

// A target represents a point where the player wants throw or shoot
// Note that the actual direction is wherever the player character is facing.
// Player need to wait for the character to turn before shooting.
#[derive(Debug, Component)]
struct Target;

#[derive(Debug, Component, PartialEq, Eq)]
pub enum Activity {
    Ducking,
    Standing,
    Walking,
    Running,
}

#[derive(Debug, Component)]
struct ShotAudio;

#[derive(Debug, Component)]
pub struct Bullet;

fn label_player(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let player_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "Player").then_some(ent))
        .unwrap();

    commands
        .entity(player_ent)
        .insert(Player::default())
        .insert(Activity::Ducking);

    let player_interact_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "InteractVolume").then_some(ent))
        .unwrap();
    commands
        .entity(player_interact_ent)
        .insert(PlayerInteractVolume);
}

fn label_shot_audio(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let goal = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "ShotAudio").then_some(ent))
        .unwrap();

    commands.entity(goal).insert(ShotAudio);
}

fn label_goal(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let goal = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "GoToGoal").then_some(ent))
        .unwrap();

    commands.entity(goal).insert(Goal);
}

fn label_target(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let target = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AimTarget").then_some(ent))
        .unwrap();

    commands.entity(target).insert(Target);
}

fn move_player(
    mut player: Query<(&mut ErasedGodotRef, &mut Activity), With<Player>>,
    goal: Query<&Transform2D, (With<Goal>, Without<Target>)>,
    target: Query<&Transform2D, With<Target>>,
    // HACK: this system accesses the physics server and needs to be run on the
    // main thread. this system param will force this system to be run on the
    // main thread
    _scene_tree: SceneTreeRef,
) {
    let (mut player, mut activity) = player.single_mut();
    let goal = goal.single().origin;
    let target = target.single().origin;

    let physics_server = unsafe { Physics2DServer::godot_singleton() };
    let body = unsafe {
        physics_server
            .body_get_direct_state(player.get::<RigidBody2D>().get_rid())
            .unwrap()
            .assume_safe()
    };

    let goal_reached = match *activity {
        Activity::Ducking => {
            stop(body);
            return;
        }
        Activity::Standing => {
            stop(body);
            turn_toward(body, target);
            return;
        }
        Activity::Walking => {
            turn_toward(body, goal);
            advance(body, goal, WALKING_SPEED)
        }
        Activity::Running => {
            turn_toward(body, goal);
            advance(body, goal, RUNNING_SPEED)
        }
    };

    if goal_reached {
        debug!("Goal reached. Stop.");
        stop(body);
        *activity = Activity::Ducking;
        debug!("Now {activity:?}");
    }
}

fn turn_toward(body: TRef<Physics2DDirectBodyState>, goal: Vector2) {
    let transform = body.transform();

    let goal_relative_position = transform.xform_inv(goal);

    let angle = goal_relative_position.angle_to(Vector2::UP) as f64;

    let turn = -TURNING_SPEED * angle;

    body.set_angular_velocity(turn);
}

fn advance(body: TRef<Physics2DDirectBodyState>, goal: Vector2, speed: f32) -> bool {
    let transform = body.transform();

    body.set_linear_velocity(transform.basis_xform_inv(Vector2::UP) * speed);

    // Is the goal reached?
    transform.origin.distance_to(goal) < 1.0
}

fn stop(body: TRef<Physics2DDirectBodyState>) {
    body.set_linear_velocity(Vector2::ZERO);
    body.set_angular_velocity(0.0);
}

fn aim(
    mut target: Query<(&mut ErasedGodotRef, &mut Transform2D), (With<Target>, Without<Player>)>,
    mut player: Query<(&mut ErasedGodotRef, &mut Activity), (With<Player>, Without<Target>)>,
) {
    let input = Input::godot_singleton();
    let (mut player, mut activity) = player.single_mut();
    let (mut target, mut transform) = target.single_mut();
    let player = player.get::<Node2D>();

    if input.is_action_pressed("aim", false) {
        // TODO: Getting mouse position from player seems odd. Isn't there a more obvious way?
        let mouse_position = player.get_global_mouse_position();
        debug!("New target is {mouse_position:?}");
        transform.origin = mouse_position;
        target.get::<Node2D>().set_visible(true);
        *activity = Activity::Standing;
        debug!("Now {activity:?}");
    }
}

fn set_goal(
    mut goal: Query<&mut Transform2D, With<Goal>>,
    mut player: Query<(&mut ErasedGodotRef, &mut Activity), With<Player>>,
) {
    let input = Input::godot_singleton();
    let (mut player, mut activity) = player.single_mut();
    let mut goal = goal.single_mut();
    let player = player.get::<Node2D>();

    if input.is_action_just_pressed("set_goal", false) {
        // TODO: Getting mouse position from player seems odd. Isn't there a more obvious way?
        let mouse_position = player.get_global_mouse_position();
        debug!("New goal is {mouse_position:?}");
        goal.origin = mouse_position;
        *activity = match *activity {
            Activity::Ducking => Activity::Walking,
            Activity::Standing => Activity::Walking,
            Activity::Walking => Activity::Walking,
            Activity::Running => Activity::Running,
        };
        debug!("Now {activity:?}");
    }
}

fn toggle_running(mut activity: Query<&mut Activity, With<Player>>) {
    let input = Input::godot_singleton();
    let mut activity = activity.single_mut();

    if input.is_action_just_pressed("toggle_running", false) {
        *activity = match *activity {
            Activity::Ducking => Activity::Running,
            Activity::Standing => Activity::Running,
            Activity::Walking => Activity::Running,
            Activity::Running => Activity::Walking,
        };
        debug!("Now {activity:?}");
    }
}

fn toggle_ducking(mut activity: Query<&mut Activity, With<Player>>) {
    let input = Input::godot_singleton();
    let mut activity = activity.single_mut();

    if input.is_action_just_pressed("toggle_ducking", false) {
        *activity = match *activity {
            Activity::Ducking => Activity::Standing,
            Activity::Standing => Activity::Ducking,
            Activity::Walking => Activity::Ducking,
            Activity::Running => Activity::Ducking,
        };
        debug!("Now {activity:?}");
    }
}

fn player_shoot(
    mut commands: Commands,
    mut target: Query<&mut ErasedGodotRef, With<Target>>,
    mut player: Query<(&mut Player, &Transform2D, &mut Activity)>,
) {
    let input = Input::godot_singleton();
    let (mut player, player_transform, mut activity) = player.single_mut();
    let mut target = target.single_mut();

    if input.is_action_just_released("aim", false) {
        debug!("Shoot!");
        let bullet_transform = *player_transform;
        commands
            .spawn()
            .insert(GodotScene::from_path("res://Bullet.tscn"))
            .insert(Bullet)
            .insert(bullet_transform);

        player.ammo_count -= 1;

        target.get::<Node2D>().set_visible(false);
        *activity = Activity::Ducking;
        debug!("Now {activity:?}");
    }
}

fn setup_bullet(
    mut bullets: Query<(&mut ErasedGodotRef, &Transform2D), Added<Bullet>>,
    mut audio: Query<&mut ErasedGodotRef, (With<ShotAudio>, Without<Bullet>)>,
) {
    for (mut bullet, bullet_transform) in bullets.iter_mut() {
        let mut audio = audio.single_mut();
        audio.get::<AudioStreamPlayer>().play(0.0);

        let bullet = bullet.get::<RigidBody2D>();
        let bullet_velocity = bullet_transform.basis_xform_inv(Vector2::new(0.0, -800.0));
        bullet.set_linear_velocity(bullet_velocity);
    }
}

fn damage_bullet(
    mut bullets: Query<(&Collisions, &mut ErasedGodotRef), With<Bullet>>,
    mut zombies: Query<&mut Hp, With<Zombie>>,
) {
    for (collisions, mut bullet) in bullets.iter_mut() {
        if collisions.recent_collisions().is_empty() {
            continue;
        }

        for collision_ent in collisions.recent_collisions() {
            let mut zombie_hp = zombies.get_mut(*collision_ent).unwrap();
            zombie_hp.0 -= 5.0;
        }

        let bullet = bullet.get::<Node>();
        bullet.queue_free();
    }
}

fn place_trap(
    mut commands: Commands,
    mut player: Query<(&mut Player, &Transform2D)>,
    selected_slot: Res<SelectedItemSlot>,
) {
    let input = Input::godot_singleton();

    if input.is_action_just_pressed("place_trap", false) {
        let (mut player, player_transform) = player.single_mut();

        if let Some(slot) = selected_slot.0 {
            let mut items = player
                .inventory
                .get_items()
                .iter()
                .filter(|(_, count)| **count > 0)
                .skip(slot as usize);
            if let Some((item, _count)) = items.next().map(|(item, count)| (*item, *count)) {
                player.inventory.use_item(&item);

                commands
                    .spawn()
                    .insert(GodotScene::from_path(item.scene_path()))
                    .insert(Transform2D(
                        GodotTransform2D::IDENTITY.translated(player_transform.origin),
                    ));
            }
        }
    }
}

fn on_restart(mut player: Query<&mut Player>) {
    let mut player = player.single_mut();
    player.reset();
}
