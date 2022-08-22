use crate::player::PlayerInteractVolume;
use bevy_godot::prelude::{
    bevy_prelude::{Added, With},
    *,
};

pub struct AirDropsPlugin;
impl Plugin for AirDropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_airdrops).add_system(collect_airdrops);
    }
}

#[derive(Component)]
pub struct AirDrop;

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
    mut airdrops: Query<(&AirDrop, &mut ErasedGodotRef)>,
) {
    let player_interact_volume = player_interact_volume.single();

    for ent in player_interact_volume.recent_collisions() {
        if let Ok((_air_drop, mut reference)) = airdrops.get_mut(*ent) {
            let reference = reference.get::<Node>();
            reference.queue_free();
        }
    }
}
