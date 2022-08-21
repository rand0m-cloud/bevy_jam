use std::f32::consts::PI;

use crate::{
    player::Player,
    Hp,
};
use bevy_godot::prelude::{
    bevy_prelude::*,
    godot_prelude::Vector2,
    *,
};
use rand::prelude::*;

pub struct ZombiesPlugin;
impl Plugin for ZombiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_zombies)
            .add_system(zombies_move.as_physics_system())
            .add_system(kill_zombies.as_physics_system())
            .add_system(zombie_targeting.as_physics_system());
    }
}

#[derive(Debug, Component)]
pub struct Zombie;

// A target represents a point where a zombie wants to be
#[derive(Debug, Component)]
struct Target(Vector2);

impl Target {
    fn random() -> Self {
        let mut rng = thread_rng();
        let distance = rng.gen_range(100.0..1000.0);
        let direction = rng.gen_range(0.0..PI);
        let vector = Vector2::UP.rotated(direction) * distance;
        Self(vector)
    }
}

fn label_zombies(
    mut commands: Commands,
    entities: Query<(&Groups, Entity), Added<ErasedGodotRef>>,
) {
    for (groups, ent) in entities.iter() {
        if groups.is("zombie") {
            commands
                .entity(ent)
                .insert(Zombie)
                .insert(Hp(10.0))
                .insert(Target::random());
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
        }
        else if zombie.origin.distance_to(target.0) < 200.0 {
            // TODO: Make the new random close to the current zombie position
            *target = Target::random();
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
