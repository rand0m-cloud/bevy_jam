use crate::prelude::*;
use crate::{player::prelude::*, zombies::Zombie};
use bevy_godot::prelude::godot_prelude::RigidBody2D;
use std::f32::consts::PI;

#[derive(AssetCollection, Debug)]
pub struct WeaponAssets {
    #[asset(path = "Bullet.tscn")]
    pub bullet: Handle<GodotResource>,
}

pub struct PlayerWeaponPlugin;
impl Plugin for PlayerWeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            shoot_weapon
                .as_physics_system()
                .run_in_state(GameState::Playing),
        )
        .add_system(
            weapon_targeting
                .as_physics_system()
                .run_in_state(GameState::Playing),
        )
        .add_system(setup_bullet.as_physics_system())
        .add_system(bullet_damage.as_physics_system());
    }
}

#[derive(Component, Debug)]
pub struct Bullet;

fn shoot_weapon(
    mut commands: Commands,
    mut player: Query<(&mut PlayerInventory, &mut PlayerWeapon, &Transform2D)>,
    zombies: Query<&Transform2D, With<Zombie>>,
    weapon_assets: Res<WeaponAssets>,
    mut time: SystemDeltaTimer,
) {
    let delta = time.delta();

    let (mut inventory, mut weapon, player_transform) = player.single_mut();
    let input = Input::godot_singleton();

    weapon.reload_timer.tick(delta);

    if input.is_action_pressed("fire_weapon", false)
        && weapon.reload_timer.finished()
        && inventory.ammo_count() > 0
        && !weapon.targets.is_empty()
    {
        inventory.use_ammo(1);

        let zombie_ent = weapon.targets.pop().unwrap();
        let target = match zombies.get(zombie_ent) {
            Ok(t) => t,
            _ => return,
        };
        let dir = player_transform.origin - target.origin;
        let angle = -dir.angle() - PI / 2.0;

        let mut bullet_transform = *player_transform;
        bullet_transform.set_rotation(angle);

        weapon.fire(&mut commands, &weapon_assets, &bullet_transform);

        weapon.reload_timer.reset();
    }
}

fn weapon_targeting(
    mut player: Query<(&mut PlayerWeapon, &Transform2D)>,
    mut zombies: Query<(&Transform2D, Entity, &mut ErasedGodotRef), With<Zombie>>,
) {
    let (mut player_weapon, player) = player.single_mut();

    let mut zombies_targets = zombies
        .iter()
        .filter(|(transform, _, _)| transform.origin.distance_to(player.origin) < 500.0)
        .map(|(transform, entity, _)| (*transform, entity))
        .collect::<Vec<_>>();
    zombies_targets.sort_by(|(transform_a, _), (transform_b, _)| {
        let zombie_a = transform_a.origin.distance_to(player.origin);
        let zombie_b = transform_b.origin.distance_to(player.origin);
        zombie_a.partial_cmp(&zombie_b).unwrap()
    });

    let zombies_targets: Vec<Entity> = zombies_targets
        .into_iter()
        .map(|(_, ent)| ent)
        .take(5)
        .rev()
        .collect();

    for weapon in player_weapon.targets.iter() {
        if let Ok((_, zombie_ent, mut zombie)) = zombies.get_mut(*weapon) {
            if !zombies_targets.contains(&zombie_ent) {
                let aim_target = unsafe {
                    zombie
                        .get::<Node2D>()
                        .get_node("AimTarget")
                        .unwrap()
                        .assume_safe()
                        .cast::<Node2D>()
                        .unwrap()
                };
                aim_target.set_visible(false);
            }
        }
    }

    for zombie_ent in zombies_targets.iter() {
        let (_, _, mut zombie) = zombies.get_mut(*zombie_ent).unwrap();
        let aim_target = unsafe {
            zombie
                .get::<Node2D>()
                .get_node("AimTarget")
                .unwrap()
                .assume_safe()
                .cast::<Node2D>()
                .unwrap()
        };
        aim_target.set_visible(true);
    }

    player_weapon.targets = zombies_targets;
}

fn setup_bullet(mut bullets: Query<(&mut ErasedGodotRef, &Transform2D), Added<Bullet>>) {
    for (mut bullet, bullet_transform) in bullets.iter_mut() {
        let bullet = bullet.get::<RigidBody2D>();
        let bullet_velocity = bullet_transform.basis_xform_inv(Vector2::new(0.0, -800.0));
        bullet.set_linear_velocity(bullet_velocity);
    }
}

fn bullet_damage(
    mut bullets: Query<(&Collisions, &mut ErasedGodotRef), With<Bullet>>,
    mut zombies: Query<&mut Hp, With<Zombie>>,
) {
    for (bullet_collision, mut bullet) in bullets.iter_mut() {
        for zombie_ent in bullet_collision.colliding() {
            if let Ok(mut hp) = zombies.get_mut(*zombie_ent) {
                hp.0 -= 5.0;
                bullet.get::<Node2D>().queue_free();
                continue;
            }
        }
    }
}
