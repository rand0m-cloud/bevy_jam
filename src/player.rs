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
use std::f32::consts::PI;

const WALKING_SPEED: f32 = 40.0;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_player)
            .add_startup_system(label_shot_audio)
            .add_system(
                move_player
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(
                set_goal
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
struct Goal(Vector2);

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
        .insert(Goal(Vector2::ZERO));

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

fn move_player(
    mut player: Query<(&mut ErasedGodotRef, &Goal), With<Player>>,
    mut time: SystemDelta,
    // HACK: this system accesses the physics server and needs to be run on the
    // main thread. this system param will force this system to be run on the
    // main thread
    _scene_tree: SceneTreeRef,
) {
    let delta = time.delta_seconds();
    let (mut player, goal) = player.single_mut();

    let physics_server = unsafe { Physics2DServer::godot_singleton() };
    let direct_body_state = unsafe {
        physics_server
            .body_get_direct_state(player.get::<RigidBody2D>().get_rid())
            .unwrap()
            .assume_safe()
    };

    let mut transform = direct_body_state.transform();

    if transform.origin.distance_to(goal.0) < 10.0 {
        debug!("Goal reached. Stop.");
        direct_body_state.set_linear_velocity(Vector2::ZERO);

        return;
    }

    let goal_relative_position = transform.xform_inv(goal.0);

    let angle = goal_relative_position.angle_to(Vector2::UP);

    let turn = if angle.abs() < 0.05 {
        0.0
    } else if goal_relative_position.x >= 0.0 {
        1.0
    } else {
        -1.0
    };

    let rotation = transform.rotation();
    transform.set_rotation(rotation + 2.0 * turn * delta);

    direct_body_state.set_linear_velocity(transform.basis_xform_inv(Vector2::UP) * WALKING_SPEED);
    direct_body_state.set_angular_velocity(0.0);
    direct_body_state.set_transform(transform);
}

fn set_goal(mut player: Query<(&mut ErasedGodotRef, &mut Goal), With<Player>>) {
    let input = Input::godot_singleton();
    let (mut player, mut goal) = player.single_mut();
    let player = player.get::<Node2D>();

    if input.is_action_just_pressed("set_goal", false) {
        let mouse_position = player.get_global_mouse_position();
        debug!("New goal is {mouse_position:?}");
        goal.0 = mouse_position;
    }
}

fn player_shoot(
    mut commands: Commands,
    mut player: Query<(&mut Player, &mut ErasedGodotRef, &Transform2D)>,
) {
    let input = Input::godot_singleton();
    let (mut player, mut player_reference, player_transform) = player.single_mut();
    let player_reference = player_reference.get::<Node2D>();

    if input.is_action_just_pressed("fire_weapon", false) && player.ammo_count > 0 {
        let mouse_dir = player_reference.get_local_mouse_position().normalized();

        let mut bullet_transform = *player_transform;
        bullet_transform.origin = player_transform.xform(mouse_dir * 50.0);

        let bullet_rotation = bullet_transform.rotation() - mouse_dir.angle() + PI / 2.0;
        bullet_transform.set_rotation(bullet_rotation);

        commands
            .spawn()
            .insert(GodotScene::from_path("res://Bullet.tscn"))
            .insert(Bullet)
            .insert(bullet_transform);

        player.ammo_count -= 1;
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
