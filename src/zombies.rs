use std::f32::consts::PI;

use crate::{
    player::{Player, PlayerInteractVolume},
    GameState, Hp, Score,
};
use bevy_godot::prelude::{bevy_prelude::*, godot_prelude::Vector2, *};
use iyes_loopless::prelude::*;
use rand::prelude::*;

pub struct ZombiesPlugin;
impl Plugin for ZombiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(10.0, true)))
            .add_startup_system(populate)
            .add_system(zombie_bites.run_in_state(GameState::Playing))
            .add_system(spawn_zombies.as_physics_system())
            .add_system(zombies_move.as_physics_system())
            .add_system(despawn_faraway_zombies.as_physics_system())
            .add_system(kill_zombies.as_physics_system())
            .add_system(zombie_targeting.as_physics_system())
            .add_exit_system(GameState::GameOver, on_restart);
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

fn populate(mut commands: Commands, player: Query<&Transform2D, With<Player>>) {
    let player_origin = player
        .get_single()
        .map(|transform| transform.origin)
        .unwrap_or_default();

    for _ in 1..100 {
        let origin = random_displacement(1000, 3000) + player_origin;
        spawn_zombie(&mut commands, origin);
    }
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
        let origin = player.origin + random_displacement(1250, 3000);

        spawn_zombie(&mut commands, origin);
    }
}

fn spawn_zombie(commands: &mut Commands, origin: Vector2) {
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

fn despawn_faraway_zombies(
    player: Query<&Transform2D, With<Player>>,
    mut zombies: Query<(&Transform2D, &mut ErasedGodotRef), With<Zombie>>,
) {
    let player = player.single();
    for (transform, mut zombie) in zombies.iter_mut() {
        let distance = transform.origin.distance_to(player.origin);
        if distance > 5000.0 {
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
    mut zombies: Query<(&Target, &mut ErasedGodotRef), (With<Zombie>, Without<Player>)>,
    mut time: SystemDelta,
    // HACK: this system accesses the physics server and needs to be run on the
    // main thread. this system param will force this system to be run on the
    // main thread
    _scene_tree: SceneTreeRef,
    state: Res<CurrentState<GameState>>,
) {
    let delta = time.delta_seconds();
    for (Target(target), mut reference) in zombies.iter_mut() {
        let physics_server = unsafe { Physics2DServer::godot_singleton() };
        let direct_body_state = unsafe {
            physics_server
                .body_get_direct_state(reference.get::<RigidBody2D>().get_rid())
                .unwrap()
                .assume_safe()
        };

        if state.0 != GameState::Sheltered {
            let mut transform = direct_body_state.transform();

            let target_relative_position = transform.xform_inv(*target);
            let turn = if target_relative_position.x >= 0.0 {
                1.0
            } else {
                -1.0
            };

            let rotation = transform.rotation();
            transform.set_rotation(rotation + 1.0 * turn * delta);

            direct_body_state.set_linear_velocity(transform.basis_xform_inv(Vector2::UP) * 70.0);
            direct_body_state.set_angular_velocity(0.0);
            direct_body_state.set_transform(transform);
        } else {
            direct_body_state.set_linear_velocity(Vector2::ZERO);
            direct_body_state.set_angular_velocity(0.0);
        }
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

fn kill_zombies(
    mut zombies: Query<(&Hp, &mut ErasedGodotRef), With<Zombie>>,
    mut score: ResMut<Score>,
) {
    for (hp, mut zombie) in zombies.iter_mut() {
        if hp.0 <= 0.0 {
            let zombie = zombie.get::<Node>();
            zombie.queue_free();

            score.0 += 100;
        }
    }
}

fn zombie_bites(
    player_interact_volume: Query<&Collisions, With<PlayerInteractVolume>>,
    zombies: Query<(), With<Zombie>>,
    mut commands: Commands,
) {
    let player_interact_volume = player_interact_volume.single();

    for ent in player_interact_volume.recent_collisions() {
        if zombies.get(*ent).is_ok() {
            commands.insert_resource(NextState(GameState::GameOver));
            debug!("You got bitten!");

            break;
        }
    }
}

fn on_restart(
    commands: Commands,
    mut zombies: Query<&mut ErasedGodotRef, With<Zombie>>,
    player: Query<&Transform2D, With<Player>>,
) {
    for mut zombie in zombies.iter_mut() {
        zombie.get::<Node>().queue_free();
    }

    populate(commands, player);
}
