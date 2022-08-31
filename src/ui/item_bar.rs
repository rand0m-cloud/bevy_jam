use crate::player::prelude::*;
use crate::prelude::*;
use bevy_godot::prelude::godot_prelude::{
    Color, ColorRect, Input, Label, Null, Texture, TextureRect,
};

pub struct ItemBarUiPlugin;

impl Plugin for ItemBarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_item_bar_nodes)
            .add_system(
                update_item_bar
                    .as_visual_system()
                    .run_not_in_state(GameState::Loading),
            )
            .add_system(
                update_selected_slot
                    .as_visual_system()
                    .run_in_state(GameState::Playing),
            );
    }
}

#[derive(NodeTreeView, Component)]
pub struct ItemSlotUi {
    #[node("Background")]
    bg: ErasedGodotRef,

    #[node("ItemTexture")]
    texture: ErasedGodotRef,

    #[node("Control/ItemCounterLabel")]
    counter: ErasedGodotRef,
}

#[derive(Component)]
pub struct ItemSlot(u16);

fn label_item_bar_nodes(
    mut commands: Commands,
    mut entities: Query<(&Name, Entity, &mut ErasedGodotRef)>,
) {
    for (name, ent, mut reference) in entities.iter_mut() {
        if name.as_str().starts_with("ItemSlot") {
            let slot_num = &name["ItemSlot".len()..].parse::<u16>().unwrap() - 1;
            let ui = ItemSlotUi::from_node(reference.get::<Node>());
            commands.entity(ent).insert(ui).insert(ItemSlot(slot_num));
        }
    }
}

fn update_item_bar(
    player_inventory: Query<&PlayerInventory, Changed<PlayerInventory>>,
    mut item_slots: Query<(&ItemSlot, &mut ItemSlotUi)>,
    crafting_assets: Res<CraftingAssets>,
    assets: Res<Assets<GodotResource>>,
) {
    if let Ok(player_inventory) = player_inventory.get_single() {
        let mut player_items = player_inventory
            .get_items()
            .iter()
            .filter(|(_, count)| **count > 0)
            .collect::<Vec<_>>();
        player_items.sort();

        let mut item_slots = item_slots.iter_mut().collect::<Vec<_>>();
        item_slots.sort_by(|(item_slot_a, _), (item_slot_b, _)| item_slot_a.0.cmp(&item_slot_b.0));

        for (item_slot, ui) in item_slots.iter_mut() {
            let (count, texture) = match player_items.get(item_slot.0 as usize) {
                Some((item, count)) => {
                    let texture_handle = item.as_texture_handle(&crafting_assets);
                    let texture = assets
                        .get(texture_handle)
                        .and_then(|t| t.0.clone().cast::<Texture>())
                        .unwrap();

                    (count.to_string(), Some(texture))
                }
                _ => ("".to_string(), None),
            };

            match texture {
                Some(t) => ui.texture.get::<TextureRect>().set_texture(t),
                _ => ui.texture.get::<TextureRect>().set_texture(Null::null()),
            };

            ui.counter.get::<Label>().set_text(count);
        }
    }
}

fn update_selected_slot(
    mut selected_slot: ResMut<SelectedItemSlot>,
    mut item_slots: Query<(&ItemSlot, &mut ItemSlotUi)>,
) {
    let input = Input::godot_singleton();

    let slots = [("slot1", 0), ("slot2", 1), ("slot3", 2), ("slot4", 3)];
    let mut slot_num = selected_slot.0;

    for (action, id) in slots {
        if input.is_action_just_pressed(action, false) {
            slot_num = Some(id);
            debug!("setting slot to {}", id);
            break;
        }
    }

    selected_slot.0 = slot_num;

    let slot_num_delta = if input.is_action_just_pressed("prev_slot", false) {
        Some(-1)
    } else if input.is_action_just_pressed("next_slot", false) {
        Some(1)
    } else {
        None
    };

    let mut item_slots = item_slots.iter_mut().collect::<Vec<_>>();
    item_slots.sort_by(|(item_slot_a, _), (item_slot_b, _)| item_slot_a.0.cmp(&item_slot_b.0));

    if let Some(slot_num_delta) = slot_num_delta {
        let slot = selected_slot.0.unwrap_or_default();

        let new_slot = if slot == 0 && slot_num_delta < 0 {
            item_slots.len() as i16 + slot_num_delta
        } else {
            (slot as i16 + slot_num_delta) % item_slots.len() as i16
        };

        selected_slot.0 = Some(new_slot as u16);
    }

    for (slot, mut ui) in item_slots {
        let color = if selected_slot.0 == Some(slot.0) {
            Color::from_rgba(0.1, 0.09, 0.09, 1.0)
        } else {
            Color::from_rgba(0.1, 0.09, 0.09, 0.5)
        };

        ui.bg.get::<ColorRect>().set_frame_color(color);
    }
}
