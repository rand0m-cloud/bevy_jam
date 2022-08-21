use crate::Hp;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With},
    godot_prelude::Vector2,
    *,
};

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

fn label_zombies(
    mut commands: Commands,
    entities: Query<(&Groups, Entity), Added<ErasedGodotRef>>,
) {
    for (groups, ent) in entities.iter() {
        if groups.is("zombie") {
            commands.entity(ent).insert(Zombie).insert(Hp(10.0));
        }
    }
}

fn zombies_move(mut zombies: Query<&mut Transform2D, With<Zombie>>, mut time: SystemDelta) {
    let delta = time.delta_seconds();
    for mut transform in zombies.iter_mut() {
        let rotation = transform.rotation();
        transform.set_rotation(rotation + 0.2 * delta);
        transform.origin = transform.xform(Vector2::new(0., -1.) * 30.0 * delta);
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
