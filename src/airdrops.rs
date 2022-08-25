use crate::{
    crafting::Part,
    player::{Player, PlayerInteractVolume},
    GameState, Score,
};
use bevy::log::*;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With, Without},
    godot_prelude::Vector2,
    *,
};
use iyes_loopless::prelude::*;
use std::{f32::consts::PI, fmt::Write};

pub struct AirDropsPlugin;
impl Plugin for AirDropsPlugin {
    fn build(&self, app: &mut App) {
        // start with 50% progress on airdrop
        let mut airdrop_timer = AirDropTimer(Timer::from_seconds(10.0, false));
        airdrop_timer.0.tick(airdrop_timer.0.duration() / 2);

        app.add_startup_system(label_air_drop_indicator)
            .add_startup_system(label_air_drop_progressbar)
            .add_startup_system(label_pickup_text)
            .add_system(label_airdrops)
            .add_system(collect_airdrops)
            .add_system(drop_airdrops)
            .add_system(airdrop_indicator.as_visual_system())
            .add_system(item_pickup_text.as_visual_system())
            .insert_resource(airdrop_timer)
            .insert_resource(ItemPickupTextTimer(Timer::from_seconds(4.0, false)))
            .add_exit_system(GameState::GameOver, on_restart);
    }
}

#[derive(Component)]
pub struct AirDrop(Vec<Part>);

#[derive(Component)]
pub struct AirDropIndicator;

#[derive(Component)]
pub struct AirDropProgressBar;

#[derive(Component)]
pub struct ItemPickupText;

pub struct ItemPickupTextTimer(Timer);

pub struct AirDropTimer(Timer);

fn label_air_drop_indicator(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AirdropIndicator").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(AirDropIndicator);
}

fn label_air_drop_progressbar(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AirdropProgressBar").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(AirDropProgressBar);
}

fn label_pickup_text(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "ItemPickupText").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(ItemPickupText);
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
    mut item_pickup_text: Query<&mut ErasedGodotRef, (Without<AirDrop>, With<ItemPickupText>)>,
    mut item_pickup_timer: ResMut<ItemPickupTextTimer>,
    mut score: ResMut<Score>,
) {
    let player_interact_volume = player_interact_volume.single();

    for ent in player_interact_volume.recent_collisions() {
        if let Ok((air_drop, mut reference)) = airdrops.get_mut(*ent) {
            let reference = reference.get::<Node>();
            reference.queue_free();

            let mut player = player.single_mut();
            player.inventory.add_parts(&air_drop.0);

            let mut text_label = item_pickup_text.single_mut();
            let text_label = text_label.get::<Label>();
            let mut text = text_label.text().to_string();

            let bullets = 25;

            for part in air_drop.0.iter() {
                writeln!(&mut text, "Picked up a {part:?}").unwrap();
            }
            writeln!(&mut text, "Picked up {} bullets", bullets).unwrap();

            text_label.set_text(text);

            airdrop_timer.0.reset();
            item_pickup_timer.0.reset();

            score.0 += 250;
            player.ammo_count += bullets;
        }
    }
}

fn airdrop_indicator(
    mut airdrop_indicator: Query<(&mut Transform2D, &mut ErasedGodotRef), With<AirDropIndicator>>,
    mut airdrops: Query<(&AirDrop, &mut ErasedGodotRef), Without<AirDropIndicator>>,
) {
    let (mut indicator_transform, mut indicator) = airdrop_indicator.single_mut();
    let indicator = indicator.get::<Node2D>();

    if let Ok((_air_drop, mut reference)) = airdrops.get_single_mut() {
        let reference = reference.get::<Node2D>();

        let mut airdrop_screen_origin = reference.get_global_transform_with_canvas().origin;

        let screen_size = Vector2::new(1280.0, 720.0);
        if (airdrop_screen_origin.x <= 0.0 || airdrop_screen_origin.x >= screen_size.x)
            || (airdrop_screen_origin.y <= 0.0 || airdrop_screen_origin.y >= screen_size.y)
        {
            indicator.set_visible(true);

            let offset = 40.0;

            if airdrop_screen_origin.x <= 0.0 {
                airdrop_screen_origin.x = offset;
            } else if airdrop_screen_origin.x >= screen_size.x {
                airdrop_screen_origin.x = screen_size.x - offset;
            }

            if airdrop_screen_origin.y <= 0.0 {
                airdrop_screen_origin.y = offset;
            } else if airdrop_screen_origin.y >= screen_size.y {
                airdrop_screen_origin.y = screen_size.y - offset;
            }

            indicator_transform.0 = GodotTransform2D::IDENTITY.translated(airdrop_screen_origin);
        } else {
            indicator.set_visible(false);
        }
    } else {
        indicator.set_visible(false);
    }
}

fn item_pickup_text(
    mut time: SystemDelta,
    mut text_timer: ResMut<ItemPickupTextTimer>,
    mut text: Query<&mut ErasedGodotRef, With<ItemPickupText>>,
) {
    let delta = time.delta();

    text_timer.0.tick(delta);
    if text_timer.0.just_finished() {
        let mut text = text.single_mut();
        text.get::<Label>().set_text("");
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
