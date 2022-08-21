use crate::Hp;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With},
    *,
};

pub struct ZombiesPlugin;
impl Plugin for ZombiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_zombies)
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

fn kill_zombies(mut zombies: Query<(&Hp, &mut ErasedGodotRef), With<Zombie>>) {
    for (hp, mut zombie) in zombies.iter_mut() {
        if hp.0 <= 0.0 {
            let zombie = zombie.get::<Node>();
            zombie.queue_free();
        }
    }
}
