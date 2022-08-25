use crate::player::Player;
use bevy_godot::prelude::{
    bevy_prelude::Changed,
    godot_prelude::{Color, Null},
    *,
};

pub struct ItemBarUiPlugin;

impl Plugin for ItemBarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_item_bar_nodes)
            .add_system(update_item_bar.as_visual_system())
            .add_system(update_selected_slot.as_visual_system())
            .insert_resource(SelectedItemSlot(None));
    }
}

#[derive(Component)]
pub struct ItemSlotTexture(u16);

#[derive(Component)]
pub struct ItemSlotBackground(u16);

pub struct SelectedItemSlot(Option<u16>);

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

            let item_bg_instance_id = unsafe {
                reference
                    .get::<Node>()
                    .get_node("Background")
                    .unwrap()
                    .assume_safe()
                    .get_instance_id()
            };

            let name = name.to_string();
            let slot_num = &name["ItemSlot".len()..].parse::<u16>().unwrap() - 1;

            item_textures.push((item_texture_instance_id, item_bg_instance_id, slot_num));
        }
    }

    for (texture_instance_id, bg_instance_id, slot_num) in item_textures {
        let texture_ent = entities
            .iter()
            .find_map(|(_, ent, reference)| {
                (reference.instance_id() == texture_instance_id).then_some(ent)
            })
            .unwrap();
        commands
            .entity(texture_ent)
            .insert(ItemSlotTexture(slot_num));

        let bg_ent = entities
            .iter()
            .find_map(|(_, ent, reference)| {
                (reference.instance_id() == bg_instance_id).then_some(ent)
            })
            .unwrap();
        commands.entity(bg_ent).insert(ItemSlotBackground(slot_num));
    }
}

fn update_item_bar(
    player: Query<&Player, Changed<Player>>,
    item_textures: Query<(&ItemSlotTexture, Entity)>,
    mut entities: Query<&mut ErasedGodotRef>,
) {
    if let Ok(player) = player.get_single() {
        let mut item_bar_texture_ents = item_textures
            .iter()
            .map(|(texture, ent)| (texture.0, ent))
            .collect::<Vec<_>>();
        item_bar_texture_ents.sort_by(|(a, _), (b, _)| a.cmp(b));

        let resource_loader = ResourceLoader::godot_singleton();

        let mut item_bar_count = 0;
        player
            .inventory
            .get_items()
            .iter()
            .zip(item_bar_texture_ents.iter())
            .for_each(|((item, _count), (_texture, texture_ent))| {
                item_bar_count += 1;
                let mut texture_node = entities.get_mut(*texture_ent).unwrap();
                let texture_node = texture_node.get::<TextureRect>();

                let texture = resource_loader
                    .load(item.as_texture_path(), "", false)
                    .unwrap();
                texture_node.set_texture(texture.cast::<Texture>().unwrap());
            });

        // set remaining slots to empty texture
        for (_, texture_ent) in item_bar_texture_ents.iter().skip(item_bar_count) {
            let mut texture_node = entities.get_mut(*texture_ent).unwrap();
            let texture_node = texture_node.get::<TextureRect>();
            texture_node.set_texture(Null::null());
        }
    }
}

fn update_selected_slot(
    mut selected_slot: ResMut<SelectedItemSlot>,
    item_bg: Query<(&ItemSlotBackground, Entity)>,
    mut entities: Query<&mut ErasedGodotRef>,
) {
    let input = Input::godot_singleton();

    let slots = [("slot1", 0), ("slot2", 1), ("slot3", 2), ("slot4", 3)];
    let mut slot_num = selected_slot.0;

    for (action, id) in slots {
        if input.is_action_just_pressed(action, false) {
            slot_num = Some(id);
            println!("setting slot to {}", id);
            break;
        }
    }

    selected_slot.0 = slot_num;

    let mut item_bar_bg_ents = item_bg
        .iter()
        .map(|(bg, ent)| (bg.0, ent))
        .collect::<Vec<_>>();
    item_bar_bg_ents.sort_by(|(a, _), (b, _)| a.cmp(b));

    // set active/inactive on item slots
    for (i, bg_ent) in item_bar_bg_ents {
        let color = if selected_slot.0 == Some(i) {
            Color::from_rgba(0.1, 0.09, 0.09, 1.0)
        } else {
            Color::from_rgba(0.1, 0.09, 0.09, 0.5)
        };

        let mut bg_node = entities.get_mut(bg_ent).unwrap();
        let bg_node = bg_node.get::<ColorRect>();
        bg_node.set_frame_color(color);
    }
}
