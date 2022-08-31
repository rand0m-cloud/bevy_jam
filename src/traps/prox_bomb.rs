use crate::{
    airdrops::{AirDrop, BonusAirDrop},
    crafting::Part,
    player::prelude::*,
    prelude::*,
    ui::text_log::ItemLogEvent,
};
use bevy_godot::prelude::godot_prelude::{AnimationPlayer, Vector2};
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
    node: ProximityBombNode,
}

#[derive(Debug, Component, NodeTreeView)]
pub struct ProximityBombNode {
    #[node("AnimationPlayer")]
    animation_player: ErasedGodotRef,
}

fn label_proximity_bombs(
    mut commands: Commands,
    mut entities: Query<(&Groups, &mut ErasedGodotRef, Entity), Added<ErasedGodotRef>>,
) {
    for (groups, mut reference, ent) in entities.iter_mut() {
        if groups.is("proximity_bomb") {
            commands.entity(ent).insert(ProximityBomb {
                detonate_timer: None,
                lifetime_timer: None,
                node: ProximityBombNode::from_node(reference.get::<Node>()),
            });
        }
    }
}

fn process_proximity_bombs(
    mut commands: Commands,
    state: Res<CurrentState<GameState>>,
    mut bombs: Query<(&mut ProximityBomb, &Collisions, &mut ErasedGodotRef)>,
    mut entities: Query<&mut Hp>,
    player: Query<&Transform2D, With<Player>>,
    mut time: SystemDeltaTimer,
    mut log: EventWriter<ItemLogEvent>,
) {
    let delta = time.delta();
    if state.0 != GameState::Playing {
        return;
    }

    for (mut bomb, collisions, mut reference) in bombs.iter_mut() {
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
                        .insert(AirDrop(iter::repeat_with(Part::random).take(20).collect()))
                        .insert(BonusAirDrop)
                        .insert(airdrop_transform);
                }
            }
        } else if !collisions.recent_collisions().is_empty() {
            info!("proximity bomb is armed");

            bomb.node
                .animation_player
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
