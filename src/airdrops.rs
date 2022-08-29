use crate::{
    crafting::Part,
    player::{Player, PlayerInteractVolume},
    ui::text_log::ItemLogEvent,
    GameState, Score,
};
use bevy::log::*;
use bevy_godot::prelude::{
    bevy_prelude::{Added, EventWriter, With, Without},
    godot_prelude::Vector2,
    *,
};
use iyes_loopless::prelude::*;
use std::f32::consts::PI;

pub struct AirDropsPlugin;
impl Plugin for AirDropsPlugin {
    fn build(&self, app: &mut App) {
        // start with 50% progress on airdrop
        let mut airdrop_timer = AirDropTimer(Timer::from_seconds(10.0, false));
        airdrop_timer.0.tick(airdrop_timer.0.duration() / 2);

        app.add_startup_system(label_air_drop_indicator)
            .add_startup_system(label_air_drop_progressbar)
            .add_system(label_airdrops)
            .add_system(collect_airdrops)
            .add_system(drop_airdrops)
            .add_system(airdrop_indicator.as_visual_system())
            .insert_resource(airdrop_timer)
            .add_exit_system(GameState::GameOver, on_restart);
    }
}

#[derive(Component)]
pub struct AirDrop(Vec<Part>);

#[derive(Component)]
pub struct AirDropIndicator;

#[derive(Component)]
pub struct AirDropIndicatorLabel;

#[derive(Component)]
pub struct AirDropProgressBar;

pub struct AirDropTimer(Timer);

fn label_air_drop_indicator(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AirdropIndicator").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(AirDropIndicator);

    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AirdropDistance").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(AirDropIndicatorLabel);
}

fn label_air_drop_progressbar(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AirdropProgressBar").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(AirDropProgressBar);
}

fn label_airdrops(
    mut commands: Commands,
    entities: Query<(&Groups, Entity), Added<ErasedGodotRef>>,
) {
    for (groups, ent) in entities.iter() {
        if groups.is("airdrop") {
            commands.entity(ent).insert(AirDrop(vec![
                Part::random(),
                Part::random(),
                Part::random(),
            ]));
        }
    }
}

fn drop_airdrops(
    mut commands: Commands,
    mut time: SystemDelta,
    mut airdrop_timer: ResMut<AirDropTimer>,
    player: Query<&Transform2D, With<Player>>,
    mut progress_bar: Query<&mut ErasedGodotRef, With<AirDropProgressBar>>,
    state: Res<CurrentState<GameState>>,
) {
    let delta = time.delta();

    if state.0 != GameState::Playing {
        return;
    }

    airdrop_timer.0.tick(delta);

    let mut progress_bar = progress_bar.single_mut();
    progress_bar
        .get::<ProgressBar>()
        .set_value(airdrop_timer.0.percent() as f64);

    if airdrop_timer.0.just_finished() {
        let mut airdrop_transform = *player.single();

        airdrop_transform.set_rotation(rand::random::<f32>() * 2.0 * PI);
        airdrop_transform.0 = airdrop_transform.translated(Vector2::UP * 1500.0);
        airdrop_transform.set_rotation(0.0);

        info!("dropping airdrop at {:?}", airdrop_transform.origin);

        commands
            .spawn()
            .insert(GodotScene::from_path("res://Airdrop.tscn"))
            .insert(airdrop_transform);
    }
}

fn collect_airdrops(
    player_interact_volume: Query<&Collisions, With<PlayerInteractVolume>>,
    mut player: Query<&mut Player>,
    mut airdrops: Query<(&AirDrop, &mut ErasedGodotRef)>,
    mut airdrop_timer: ResMut<AirDropTimer>,
    mut item_log: EventWriter<ItemLogEvent>,
    mut score: ResMut<Score>,
) {
    let player_interact_volume = player_interact_volume.single();

    for ent in player_interact_volume.recent_collisions() {
        if let Ok((air_drop, mut reference)) = airdrops.get_mut(*ent) {
            let reference = reference.get::<Node>();
            reference.queue_free();

            let mut player = player.single_mut();
            player.inventory.add_parts(&air_drop.0);

            let bullets = 25;

            for part in air_drop.0.iter() {
                item_log.send(ItemLogEvent(format!("Picked up a {part:?}")));
            }
            item_log.send(ItemLogEvent(format!("Picked up {} bullets", bullets)));

            airdrop_timer.0.reset();

            score.0 += 250;
            player.ammo_count += bullets;
        }
    }
}

fn airdrop_indicator(
    mut airdrop_indicator: Query<
        (&mut Transform2D, &mut ErasedGodotRef),
        (
            With<AirDropIndicator>,
            Without<AirDropIndicatorLabel>,
            Without<AirDrop>,
            Without<Player>,
        ),
    >,
    mut airdrop_indicator_label: Query<
        &mut ErasedGodotRef,
        (
            With<AirDropIndicatorLabel>,
            Without<AirDrop>,
            Without<AirDropIndicator>,
        ),
    >,
    mut airdrops: Query<
        (&Transform2D, &mut ErasedGodotRef),
        (
            With<AirDrop>,
            Without<AirDropIndicator>,
            Without<AirDropIndicatorLabel>,
        ),
    >,
    player: Query<&Transform2D, With<Player>>,
) {
    let (mut indicator_transform, mut indicator) = airdrop_indicator.single_mut();
    let indicator = indicator.get::<Node2D>();

    let player = player.single();

    if let Ok((air_drop_transform, mut air_drop)) = airdrops.get_single_mut() {
        let mut airdrop_screen_origin = {
            let air_drop = air_drop.get::<Node2D>();
            air_drop.get_global_transform_with_canvas().origin
        };

        let mut indicator_label = airdrop_indicator_label.single_mut();
        let indicator_label = indicator_label.get::<Label>();

        let screen_size = Vector2::new(1280.0, 720.0);

        // calculate the indicator's origin and keep the offset used
        let indicator_origin_and_offset = if (airdrop_screen_origin.x <= 0.0
            || airdrop_screen_origin.x >= screen_size.x)
            || (airdrop_screen_origin.y <= 0.0 || airdrop_screen_origin.y >= screen_size.y)
        {
            let offset = 40.0;
            let mut offset_vector2 = Vector2::ZERO;

            if airdrop_screen_origin.x <= 0.0 {
                airdrop_screen_origin.x = offset;
                offset_vector2.x = offset;
            } else if airdrop_screen_origin.x >= screen_size.x {
                airdrop_screen_origin.x = screen_size.x - offset;
                offset_vector2.x = -offset;
            }

            if airdrop_screen_origin.y <= 0.0 {
                airdrop_screen_origin.y = offset;
                offset_vector2.y = offset;
            } else if airdrop_screen_origin.y >= screen_size.y {
                airdrop_screen_origin.y = screen_size.y - offset;
                offset_vector2.y = -offset;
            }

            Some((airdrop_screen_origin, offset_vector2))
        } else {
            None
        };

        if let Some((origin, offset)) = indicator_origin_and_offset {
            indicator_transform.0 = GodotTransform2D::IDENTITY.translated(origin);
            indicator.set_visible(true);

            let distance = air_drop_transform.origin.distance_to(player.origin);
            indicator_label.set_position(offset, false);
            indicator_label.set_text(format!("{:.0}m", distance / 8.0));
        } else {
            indicator.set_visible(false);
        }
    } else {
        indicator.set_visible(false);
    }
}

fn on_restart(
    mut airdrops: Query<&mut ErasedGodotRef, With<AirDrop>>,
    mut airdrop_timer: ResMut<AirDropTimer>,
) {
    for mut airdrop in airdrops.iter_mut() {
        airdrop.get::<Node>().queue_free();
    }

    airdrop_timer.0.reset();
    let timer_duration = airdrop_timer.0.duration();
    airdrop_timer.0.tick(timer_duration / 2);
}
