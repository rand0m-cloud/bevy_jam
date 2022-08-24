use crate::{
    crafting::Part,
    player::{Player, PlayerInteractVolume},
};
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
            .add_system(airdrop_indicator.as_visual_system());
    }
}

#[derive(Component)]
pub struct AirDrop;

#[derive(Component)]
pub struct AirDropIndicator;

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
            commands.entity(ent).insert(AirDrop);
        }
    }
}

fn collect_airdrops(
    player_interact_volume: Query<&Collisions, With<PlayerInteractVolume>>,
    mut player: Query<&mut Player>,
    mut airdrops: Query<(&AirDrop, &mut ErasedGodotRef)>,
) {
    let player_interact_volume = player_interact_volume.single();

    for ent in player_interact_volume.recent_collisions() {
        if let Ok((_air_drop, mut reference)) = airdrops.get_mut(*ent) {
            let reference = reference.get::<Node>();
            reference.queue_free();

            let mut player = player.single_mut();
            let part = Part::random();

            player.inventory.add_part(part);
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
    }
}
