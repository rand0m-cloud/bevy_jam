use std::f32::consts::PI;

use crate::{player::Player, Hp};
use bevy_godot::prelude::{bevy_prelude::*, godot_prelude::Vector2, *};
use rand::prelude::*;

pub struct ZombiesPlugin;
impl Plugin for ZombiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(10.0, true)))
            .add_system(spawn_zombies.as_physics_system())
            .add_system(zombies_move.as_physics_system())
            .add_system(despawn_faraway_zombies.as_physics_system())
            .add_system(kill_zombies.as_physics_system())
            .add_system(zombie_targeting.as_physics_system());
    }
}

#[derive(Debug, Component)]
pub struct Zombie;

struct SpawnTimer(Timer);

// A target represents a point where a zombie wants to be
#[derive(Debug, Component)]
struct Target(Vector2);

impl Target {
    fn random(origin: Vector2) -> Self {
        let vector = random_displacement(100, 1000);
        Self(vector + origin)
    }
}

fn random_displacement(min_distance: u32, max_distance: u32) -> Vector2 {
    let mut rng = thread_rng();
    let range = (min_distance as f32)..(max_distance as f32);
    let distance = rng.gen_range(range);
    let direction = rng.gen_range(0.0..(2.0 * PI));
    Vector2::UP.rotated(direction) * distance
}

fn spawn_zombies(
    mut commands: Commands,
    player: Query<&Transform2D, With<Player>>,
    zombies: Query<(), With<Zombie>>,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        // Limit spawning rate if population is large
        let population_target = 200.0;
        let actual_population = zombies.iter().count() as f32;
        let probability = population_target / 100.0 / actual_population.sqrt();
        debug!("Current population is {actual_population}");
        if random::<f32>() > probability {
            return;
        };

        // Spawn new zombie away from the player
        let player = player.single();
        let origin = player.origin + random_displacement(10000, 50000);

        debug!("Spawning at {origin:?}");
        commands
            .spawn()
            .insert(GodotScene::from_path("res://Zombie.tscn"))
            .insert(Zombie)
            .insert(Hp(10.0))
            .insert(Target::random(origin))
            .insert(Transform2D(
                GodotTransform2D::from_rotation_translation_scale(origin, 0.0, Vector2::ONE),
            ));
    }
}

fn despawn_faraway_zombies(
    player: Query<&Transform2D, With<Player>>,
    mut zombies: Query<(&Transform2D, &mut ErasedGodotRef), With<Zombie>>,
) {
    let player = player.single();
    for (transform, mut zombie) in zombies.iter_mut() {
        let distance = transform.origin.distance_to(player.origin);
        if distance > 60000.0 {
            debug!(
                "{:?} is too far from {:?} ({:?}). Despawning.",
                transform.origin, player.origin, distance
            );
            let zombie = zombie.get::<Node>();
            zombie.queue_free();
        }
    }
}

fn zombies_move(
    mut zombies: Query<(&mut Transform2D, &Target), (With<Zombie>, Without<Player>)>,
    mut time: SystemDelta,
) {
    let delta = time.delta_seconds();
    for (mut zombie, Target(target)) in zombies.iter_mut() {
        let target_relative_position = zombie.xform_inv(*target);

        let turn = if target_relative_position.x >= 0.0 {
            1.0
        } else {
            -1.0
        };

        let rotation = zombie.rotation();
        zombie.set_rotation(rotation + 0.5 * turn * delta);

        // Move forward
        zombie.origin = zombie.xform(Vector2::new(0., -1.) * 30.0 * delta);
    }
}

fn zombie_targeting(
    mut zombies: Query<(&Transform2D, &mut Target), With<Zombie>>,
    player: Query<&Transform2D, With<Player>>,
) {
    for (zombie, mut target) in zombies.iter_mut() {
        let player = player.single();
        if zombie.origin.distance_to(player.origin) < 500.0 {
            *target = Target(player.origin);
        } else if zombie.origin.distance_to(target.0) < 200.0 {
            *target = Target::random(zombie.origin);
        }
    }
}

fn kill_zombies(mut zombies: Query<(&Hp, &mut ErasedGodotRef), With<Zombie>>) {
    for (hp, mut zombie) in zombies.iter_mut() {
        if hp.0 <= 0.0 {
            let zombie = zombie.get::<Node>();
            zombie.queue_free();
        }
    }
}
