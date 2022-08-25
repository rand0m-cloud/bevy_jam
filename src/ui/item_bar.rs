use crate::player::Player;
use bevy_godot::prelude::{bevy_prelude::Changed, godot_prelude::Null, *};

pub struct ItemBarUiPlugin;

impl Plugin for ItemBarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_item_bar_nodes)
            .add_system(update_item_bar.as_visual_system());
    }
}

#[derive(Component)]
pub struct ItemBarTexture(u16);

fn label_item_bar_nodes(
    mut commands: Commands,
    mut entities: Query<(&Name, Entity, &mut ErasedGodotRef)>,
) {
    let mut item_textures = vec![];
    for (name, _, mut reference) in entities.iter_mut() {
        if name.as_str().starts_with("ItemSlot") {
            let item_texture_instance_id = unsafe {
                reference
                    .get::<Node>()
                    .get_node("ItemTexture")
                    .unwrap()
                    .assume_safe()
                    .get_instance_id()
            };

            let name = name.to_string();
            let slot_num = &name["ItemSlot".len()..].parse::<u16>().unwrap() - 1;

            item_textures.push((item_texture_instance_id, slot_num));
        }
    }

    for (instance_id, slot_num) in item_textures {
        let ent = entities
            .iter()
            .find_map(|(_, ent, reference)| (reference.instance_id() == instance_id).then_some(ent))
            .unwrap();
        commands.entity(ent).insert(ItemBarTexture(slot_num));
    }
}

fn update_item_bar(
    player: Query<&Player, Changed<Player>>,
    mut items: Query<(&ItemBarTexture, &mut ErasedGodotRef, Entity)>,
) {
    if let Ok(player) = player.get_single() {
        let mut item_bar_ents = items
            .iter()
            .map(|(texture, _, ent)| (texture.0, ent))
            .collect::<Vec<_>>();
        item_bar_ents.sort_by(|(a, _), (b, _)| a.cmp(b));

        let resource_loader = ResourceLoader::godot_singleton();

        let mut item_bar_count = 0;
        player
            .inventory
            .get_items()
            .iter()
            .zip(item_bar_ents.iter())
            .for_each(|((item, _count), (_texture, texture_ent))| {
                item_bar_count += 1;
                let (_, mut texture_node, _) = items.get_mut(*texture_ent).unwrap();
                let texture_node = texture_node.get::<TextureRect>();

                let texture = resource_loader
                    .load(item.as_texture_path(), "", false)
                    .unwrap();
                texture_node.set_texture(texture.cast::<Texture>().unwrap());
            });

        // set remaining slots to empty texture
        for (_, texture_ent) in item_bar_ents.iter().skip(item_bar_count) {
            let (_, mut texture_node, _) = items.get_mut(*texture_ent).unwrap();
            let texture_node = texture_node.get::<TextureRect>();
            texture_node.set_texture(Null::null());
        }
    }
}
