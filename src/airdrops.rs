use crate::{
    crafting::Part,
    player::{Player, PlayerInteractVolume},
};
use bevy::log::*;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With, Without},
    godot_prelude::Vector2,
    *,
};

pub struct AirDropsPlugin;
impl Plugin for AirDropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_air_drop_indicator)
            .add_system(label_airdrops)
            .add_system(collect_airdrops)
            .add_system(drop_airdrops)
            .add_system(airdrop_indicator.as_visual_system())
            .insert_resource(AirDropTimer(Timer::from_seconds(5.0, false)));
    }
}

#[derive(Component)]
pub struct AirDrop(Vec<Part>);

#[derive(Component)]
pub struct AirDropIndicator;

pub struct AirDropTimer(Timer);

fn label_air_drop_indicator(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "AirdropIndicator").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(AirDropIndicator);
}

fn label_airdrops(
    mut commands: Commands,
    entities: Query<(&Groups, Entity), Added<ErasedGodotRef>>,
) {
    for (groups, ent) in entities.iter() {
        if groups.is("airdrop") {
            commands
                .entity(ent)
                .insert(AirDrop(vec![Part::random(), Part::random()]));
        }
    }
}

fn drop_airdrops(
    mut commands: Commands,
    mut time: SystemDelta,
    mut airdrop_timer: ResMut<AirDropTimer>,
    player: Query<&Transform2D, With<Player>>,
) {
    let delta = time.delta();

    airdrop_timer.0.tick(delta);
    if airdrop_timer.0.just_finished() {
        let mut airdrop_transform = *player.single();

        airdrop_transform.set_rotation(rand::random());
        airdrop_transform.0 = airdrop_transform.translated(Vector2::UP * 3000.0);
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
) {
    let player_interact_volume = player_interact_volume.single();

    for ent in player_interact_volume.recent_collisions() {
        if let Ok((air_drop, mut reference)) = airdrops.get_mut(*ent) {
            let reference = reference.get::<Node>();
            reference.queue_free();

            let mut player = player.single_mut();
            player.inventory.add_parts(&air_drop.0);

            airdrop_timer.0.reset();
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
