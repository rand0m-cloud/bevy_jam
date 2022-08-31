use crate::player::prelude::*;
use crate::prelude::*;

pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            use_trap
                .as_physics_system()
                .run_in_state(GameState::Playing),
        );
    }
}

fn use_trap(
    mut commands: Commands,
    mut player: Query<&mut PlayerInventory>,
    trap_target: Query<&Transform2D, With<PlayerTrapTarget>>,
    selected_slot: Res<SelectedItemSlot>,
    assets: Res<CraftingAssets>,
) {
    let mut player_inventory = player.single_mut();
    let trap_target = trap_target.single();

    let input = Input::godot_singleton();
    if !input.is_action_just_pressed("place_trap", false) {
        return;
    }

    let selected_slot = match selected_slot.0 {
        Some(i) => i,
        _ => return,
    };

    let mut items: Vec<Item> = player_inventory
        .get_items()
        .iter()
        .filter_map(|(item, count)| (*count > 0).then_some(*item))
        .collect();
    items.sort();

    let item = match items.get(selected_slot as usize) {
        Some(item) => item,
        _ => return,
    };

    player_inventory.use_item(item);

    let mut transform = *trap_target;
    transform.set_rotation(0.0);
    transform.set_scale(Vector2::ONE);

    commands
        .spawn()
        .insert(GodotScene::from_handle(item.as_scene_handle(&assets)))
        .insert(transform);
}
