use crate::crafting::{Inventory, Item, Part};
use crate::Hp;
use crate::{zombies::Zombie, GameState};
use bevy_godot::prelude::{
    bevy_prelude::{Added, With},
    godot_prelude::Vector2,
    *,
};
use iyes_loopless::prelude::*;
use std::f32::consts::PI;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_player)
            .add_system(
                move_player
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(
                player_shoot
                    .as_physics_system()
                    .run_in_state(GameState::Playing),
            )
            .add_system(setup_bullet.as_physics_system())
            .add_system(damage_bullet);
    }
}

#[derive(Debug, Component)]
pub struct Player {
    pub inventory: Inventory,
}

#[derive(Debug, Component)]
pub struct PlayerInteractVolume;

#[derive(Debug, Component)]
pub struct Bullet;

fn label_player(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let player_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "Player").then_some(ent))
        .unwrap();

    // setup initial inventory with parts for one alarm or drone
    let mut inventory = Inventory::default();
    inventory.add_parts(&Item::Alarm.ingredients());
    inventory.add_part(Part::Motor);

    commands.entity(player_ent).insert(Player { inventory });

    let player_interact_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "InteractVolume").then_some(ent))
        .unwrap();
    commands
        .entity(player_interact_ent)
        .insert(PlayerInteractVolume);
}

fn move_player(mut player: Query<(&Player, &mut Transform2D)>, mut time: SystemDelta) {
    let (_, mut player_transform) = player.single_mut();
    let input = Input::godot_singleton();

    let move_forward = input.get_action_strength("move_forward", false);
    let move_backward = input.get_action_strength("move_backward", false);

    let rotate_left = input.get_action_strength("rotate_left", false);
    let rotate_right = input.get_action_strength("rotate_right", false);

    let move_input = move_backward - move_forward;
    let rotate_input = rotate_right - rotate_left;

    let delta = time.delta_seconds();

    player_transform.origin =
        player_transform.xform(Vector2::new(0.0, move_input as f32) * 100.0 * delta);

    let rotation = player_transform.rotation();
    player_transform.set_rotation(rotate_input as f32 * 1.5 * delta + rotation);
}

fn player_shoot(
    mut commands: Commands,
    mut player: Query<(&mut ErasedGodotRef, &Transform2D), With<Player>>,
) {
    let input = Input::godot_singleton();
    let (mut player, player_transform) = player.single_mut();
    let player = player.get::<Node2D>();

    if input.is_action_just_pressed("fire_weapon", false) {
        let mouse_dir = player.get_local_mouse_position().normalized();

        let mut bullet_transform = *player_transform;
        bullet_transform.origin = player_transform.xform(mouse_dir * 50.0);

        let bullet_rotation = bullet_transform.rotation() - mouse_dir.angle() + PI / 2.0;
        bullet_transform.set_rotation(bullet_rotation);

        commands
            .spawn()
            .insert(GodotScene::from_path("res://Bullet.tscn"))
            .insert(Bullet)
            .insert(bullet_transform);
    }
}

fn setup_bullet(mut bullets: Query<(&mut ErasedGodotRef, &Transform2D), Added<Bullet>>) {
    for (mut bullet, bullet_transform) in bullets.iter_mut() {
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
