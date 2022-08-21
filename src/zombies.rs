use std::f32::consts::PI;

use crate::{Hp, player::Player};
use bevy::prelude::*;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With},
    godot_prelude::Vector2,
    *,
};
use rand::prelude::*;

pub struct ZombiesPlugin;
impl Plugin for ZombiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_zombies)
            .add_system(zombies_move.as_physics_system())
            .add_system(kill_zombies.as_physics_system());
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
        let distance = rng.gen_range(10.0..100.0);
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
    player: Query<&Transform2D, (With<Player>, Without<Zombie>)>,
    mut time: SystemDelta,
) {
    let delta = time.delta_seconds();
    let player = player.single();
    for (mut zombie, Target(_target)) in zombies.iter_mut() {

        let origin = zombie.origin;

        // Just for testing
        let target = &player.origin;

        info!("Zombie at {:?} wants to be at {target:?}", zombie.origin);

        // Rotate toward target
        let heading = (zombie.rotation() / 2.0 / PI);
        let desired_rotation = (origin.angle_to_point(*target) / 2.0 / PI);
        let relative_rotation = (desired_rotation - heading);

        info!("Heading:     {heading:?}");
        info!("Desired:     {desired_rotation:?}");
        info!("Relative:    {relative_rotation:?}");


        let turn = if relative_rotation > 0.0 {
            info!("Turning right");
            1.0
        } else {
            info!("Turning left");
            -1.0
        };

        // let rotation = zombie.rotation();
        // zombie.set_rotation(rotation + 0.5 * turn * delta);

        // // Move forward
        // zombie.origin = zombie.xform(Vector2::new(0., 1.) * 30.0 * delta);
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
