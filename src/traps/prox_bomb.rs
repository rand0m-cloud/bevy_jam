use crate::{
    airdrops::{AirDrop, BonusAirDrop},
    crafting::Part,
    player::Player,
    ui::text_log::ItemLogEvent,
    Hp,
};
use bevy::log::*;
use bevy_godot::prelude::{
    bevy_prelude::{Added, EventWriter, With, Without},
    godot_prelude::Vector2,
    *,
};
use std::{f32::consts::PI, iter};

pub struct ProximityBombPlugin;
impl Plugin for ProximityBombPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_proximity_bombs)
            .add_system(process_proximity_bombs);
    }
}

#[derive(Debug, Component)]
pub struct ProximityBomb {
    detonate_timer: Option<Timer>,
    lifetime_timer: Option<Timer>,
}

#[derive(Debug, Component)]
pub struct ProximityBombAnimationPlayer(Entity);

fn label_proximity_bombs(
    mut commands: Commands,
    mut entities: Query<(&Groups, &mut ErasedGodotRef, Entity), Added<ErasedGodotRef>>,
) {
    let mut animation_players = vec![];
    for (groups, mut reference, ent) in entities.iter_mut() {
        if groups.is("proximity_bomb") {
            commands.entity(ent).insert(ProximityBomb {
                detonate_timer: None,
                lifetime_timer: None,
            });

            let animation_player = unsafe {
                reference
                    .get::<Node2D>()
                    .get_node("AnimationPlayer")
                    .unwrap()
                    .assume_safe()
                    .get_instance_id()
            };

            animation_players.push((ent, animation_player));
        }
    }

    for (bomb, animation_player_instance_id) in animation_players {
        let animation_player = entities
            .iter_mut()
            .find_map(|(_, reference, ent)| {
                (reference.instance_id() == animation_player_instance_id).then_some(ent)
            })
            .unwrap();
        commands
            .entity(animation_player)
            .insert(ProximityBombAnimationPlayer(bomb));
    }
}

fn process_proximity_bombs(
    mut commands: Commands,
    mut bombs: Query<(&mut ProximityBomb, Entity, &Collisions, &mut ErasedGodotRef)>,
    mut animation_player: Query<
        (&ProximityBombAnimationPlayer, &mut ErasedGodotRef),
        Without<ProximityBomb>,
    >,
    mut entities: Query<&mut Hp>,
    player: Query<&Transform2D, With<Player>>,
    mut time: SystemDelta,
    mut log: EventWriter<ItemLogEvent>,
) {
    let delta = time.delta();

    for (mut bomb, bomb_ent, collisions, mut reference) in bombs.iter_mut() {
        if let Some(bomb_timer) = bomb.detonate_timer.as_mut() {
            bomb_timer.tick(delta);
            if bomb_timer.just_finished() {
                info!("proximity bomb went off");
                let mut killed_zombies = 0;

                for ent in collisions.colliding() {
                    if let Ok(mut obj_hp) = entities.get_mut(*ent) {
                        obj_hp.0 = 0.0;
                        killed_zombies += 1;
                    }
                }

                if killed_zombies > 5 {
                    log.send(ItemLogEvent(format!(
                        "{}x Killing Spree! An extra airdrop is on the way!",
                        killed_zombies
                    )));

                    let mut airdrop_transform = *player.single();

                    airdrop_transform.set_rotation(rand::random::<f32>() * 2.0 * PI);
                    airdrop_transform.0 = airdrop_transform.translated(Vector2::UP * 1000.0);
                    airdrop_transform.set_rotation(0.0);

                    commands
                        .spawn()
                        .insert(GodotScene::from_path("res://Airdrop.tscn"))
                        .insert(AirDrop(
                            iter::repeat_with(|| Part::random())
                                .take(5)
                                .collect::<Vec<_>>(),
                        ))
                        .insert(BonusAirDrop)
                        .insert(airdrop_transform);
                }
            }
        } else if !collisions.recent_collisions().is_empty() {
            info!("proximity bomb is armed");

            let mut animation_player = animation_player
                .iter_mut()
                .find_map(|(player, reference)| (player.0 == bomb_ent).then_some(reference))
                .unwrap();
            animation_player
                .get::<AnimationPlayer>()
                .play("detonate", -1.0, 1.0, false);

            bomb.detonate_timer = Some(Timer::from_seconds(2.0, false));
            bomb.lifetime_timer = Some(Timer::from_seconds(3.0, false));
        }

        if let Some(lifetime_timer) = bomb.lifetime_timer.as_mut() {
            lifetime_timer.tick(delta);
            if lifetime_timer.just_finished() {
                reference.get::<Node2D>().queue_free();
            }
        }
    }
}
