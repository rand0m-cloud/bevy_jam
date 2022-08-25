use crate::Hp;
use bevy::log::*;
use bevy_godot::prelude::{bevy_prelude::Added, *};

pub struct ProximityBombPlugin;
impl Plugin for ProximityBombPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_proximity_bombs)
            .add_system(process_proximity_bombs);
    }
}

#[derive(Debug, Component)]
pub struct ProximityBomb(Option<Timer>);

fn label_proximity_bombs(
    mut commands: Commands,
    entities: Query<(&Groups, Entity), Added<ErasedGodotRef>>,
) {
    for (groups, ent) in entities.iter() {
        if groups.is("proximity_bomb") {
            commands.entity(ent).insert(ProximityBomb(None));
        }
    }
}

fn process_proximity_bombs(
    mut bombs: Query<(&mut ProximityBomb, &Collisions, &mut ErasedGodotRef)>,
    mut entities: Query<&mut Hp>,
    mut time: SystemDelta,
) {
    let delta = time.delta();
    for (mut bomb, collisions, mut reference) in bombs.iter_mut() {
        if let Some(bomb_timer) = bomb.0.as_mut() {
            bomb_timer.tick(delta);
            if bomb_timer.just_finished() {
                info!("proximity bomb went off");
                for ent in collisions.colliding() {
                    let mut obj_hp = entities.get_mut(*ent).unwrap();
                    obj_hp.0 = 0.0;
                }
                reference.get::<Node>().queue_free();
            }
        } else if !collisions.recent_collisions().is_empty() {
            info!("proximity bomb is armed");
            bomb.0 = Some(Timer::from_seconds(2.0, false));
        }
    }
}
