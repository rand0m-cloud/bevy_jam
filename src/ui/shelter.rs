use crate::{
    crafting::{CraftingAssets, Item, Part},
    player::prelude::*,
    prelude::*,
};
use bevy_godot::prelude::godot_prelude::{
    Button, Color, Input, Label, RichTextLabel, Texture, TextureRect,
};
use std::fmt::Write;

pub struct ShelterUiPlugin;

impl Plugin for ShelterUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_shelter_ui)
            .add_system(debug_toggle_shelter_mode.as_visual_system())
            .add_system(listen_for_crafting_ui_presses.run_in_state(GameState::Sheltered))
            .add_system(
                listen_for_crafting_ui_actions
                    .as_visual_system()
                    .run_in_state(GameState::Sheltered),
            )
            .add_system(update_recipe_preview.run_not_in_state(GameState::Loading))
            .add_enter_system(GameState::Sheltered, enter_shelter_ui)
            .add_exit_system(GameState::Sheltered, exit_shelter_ui);
    }
}

#[derive(Component, Debug)]
struct CraftingTarget(Item);

impl CraftingTarget {
    fn next(&mut self) {
        let new_item = Item::ALL
            .iter()
            .cycle()
            .skip_while(|e| **e != self.0)
            .skip(1)
            .next()
            .unwrap();
        self.0 = *new_item;
    }

    fn previous(&mut self) {
        let new_item = Item::ALL
            .iter()
            .rev()
            .cycle()
            .skip_while(|e| **e != self.0)
            .skip(1)
            .next()
            .unwrap();
        self.0 = *new_item;
    }
}

#[derive(NodeTreeView, Component)]
pub struct ShelterUi {
    #[node("TabContainer/Crafting/MarginContainer/ScrollContainer/Craftables")]
    craftables: ErasedGodotRef,

    #[node("TabContainer/Crafting/Control/SelectedItem")]
    crafting_recipe_preview: ErasedGodotRef,

    #[node("TabContainer/Crafting/Control/SelectedItemText")]
    crafting_recipe_preview_text: ErasedGodotRef,

    #[node("TabContainer/Crafting/Control/CraftButton")]
    craft_button: ErasedGodotRef,

    #[node("TabContainer/Crafting/Control/ItemRecipeText")]
    crafting_recipe_text: ErasedGodotRef,
}

impl ShelterUi {
    pub fn refresh(&mut self, inventory: &PlayerInventory, craftables: &mut Craftables) {
        for (item, reference) in craftables.iter_mut() {
            // mark items as craftable
            let modulate_color = if inventory.can_craft(item) {
                Color::from_rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::from_rgba(1.0, 1.0, 1.0, 0.3)
            };

            reference.get::<Control>().set_modulate(modulate_color);
        }
    }
}

#[derive(NodeTreeView, Component)]
pub struct Craftables {
    #[node("ProximityBomb")]
    proximity_bomb: ErasedGodotRef,

    #[node("Alarm")]
    alarm: ErasedGodotRef,
}

impl Craftables {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Item, &mut ErasedGodotRef)> {
        [
            (Item::ProximityBomb, &mut self.proximity_bomb),
            (Item::Alarm, &mut self.alarm),
        ]
        .into_iter()
    }
}

fn setup_shelter_ui(
    mut commands: Commands,
    mut entities: Query<(&Name, (Entity, &mut ErasedGodotRef))>,
    mut scene_tree: SceneTreeRef,
) {
    let (screen, mut screen_reference) = entities
        .iter_mut()
        .find_entity_by_name("ShelterUI")
        .unwrap();
    let mut ui = ShelterUi::from_node(screen_reference.get::<Node>());
    let mut craftables = Craftables::from_node(ui.craftables.get::<Node>());

    connect_godot_signal(&mut ui.craft_button, "pressed", &mut scene_tree);

    for (_, obj) in craftables.iter_mut() {
        connect_godot_signal(obj, "pressed", &mut scene_tree);
    }

    commands
        .entity(screen)
        .insert(ui)
        .insert(craftables)
        .insert(CraftingTarget(Item::ProximityBomb));
}

fn enter_shelter_ui(
    mut screen: Query<(&mut ErasedGodotRef, &mut ShelterUi, &mut Craftables)>,
    player_inventory: Query<&PlayerInventory>,
    mut scene_tree: SceneTreeRef,
) {
    debug!("Showing shelter ui.");
    let (mut screen, mut shelter_ui, mut craftables) = screen.single_mut();
    screen.get::<Control>().set_visible(true);

    shelter_ui.refresh(player_inventory.single(), &mut craftables);

    scene_tree.get().set_pause(true);
}

fn exit_shelter_ui(
    mut screen: Query<&mut ErasedGodotRef, With<ShelterUi>>,
    mut scene_tree: SceneTreeRef,
) {
    debug!("Hiding shelter ui.");
    let mut screen = screen.single_mut();
    screen.get::<Control>().set_visible(false);

    scene_tree.get().set_pause(false);
}

fn listen_for_crafting_ui_presses(
    mut events: EventReader<GodotSignal>,
    mut player_inventory: Query<&mut PlayerInventory>,
    mut shelter_ui: Query<(&mut ShelterUi, &mut Craftables, &mut CraftingTarget)>,
) {
    let mut player_inventory = player_inventory.single_mut();
    let (mut shelter_ui, mut craftables, mut crafting_target) = shelter_ui.single_mut();

    for event in events.iter() {
        if event.name() == "pressed" {
            let node_name = event.origin().get::<Node>().name().to_string();

            if let Ok(item) = Item::from_str(&node_name) {
                crafting_target.0 = item;

                shelter_ui
                    .craft_button
                    .get::<Button>()
                    .set_disabled(!player_inventory.can_craft(item));

                shelter_ui
                    .crafting_recipe_preview_text
                    .get::<Label>()
                    .set_text(item.as_str());
            } else if node_name == "CraftButton" {
                let target = crafting_target.0;
                debug!("trying to craft: {:?}", target);

                player_inventory.craft(target);

                shelter_ui
                    .craft_button
                    .get::<Button>()
                    .set_disabled(!player_inventory.can_craft(target));

                shelter_ui.refresh(&player_inventory, &mut craftables);
            }
        }
    }
}

fn listen_for_crafting_ui_actions(
    mut player_inventory: Query<&mut PlayerInventory>,
    mut shelter_ui: Query<(&mut ShelterUi, &mut Craftables, &mut CraftingTarget)>,
) {
    let (mut shelter_ui, mut craftables, mut crafting_target) = shelter_ui.single_mut();
    let mut player_inventory = player_inventory.single_mut();
    let input = Input::godot_singleton();

    if input.is_action_just_pressed("ui_right", false) {
        crafting_target.next();
    } else if input.is_action_just_pressed("ui_left", false) {
        crafting_target.previous();
    } else if input.is_action_just_pressed("ui_accept", false) {
        let item = crafting_target.0;

        player_inventory.craft(item);

        shelter_ui
            .craft_button
            .get::<Button>()
            .set_disabled(!player_inventory.can_craft(item));

        shelter_ui.refresh(&player_inventory, &mut craftables);
    } else {
        return;
    }

    let item = crafting_target.0;

    shelter_ui
        .craft_button
        .get::<Button>()
        .set_disabled(!player_inventory.can_craft(item));

    shelter_ui
        .crafting_recipe_preview_text
        .get::<Label>()
        .set_text(item.as_str());
}

fn debug_toggle_shelter_mode(mut commands: Commands, state: Res<CurrentState<GameState>>) {
    let input = Input::godot_singleton();

    if input.is_action_just_pressed("debug_enter_shelter_mode", false) {
        if state.0 == GameState::Playing {
            commands.insert_resource(NextState(GameState::Sheltered));
        } else if state.0 == GameState::Sheltered {
            commands.insert_resource(NextState(GameState::Playing));
        }
    }

    if input.is_action_just_pressed("ui_cancel", false) && state.0 == GameState::Sheltered {
        commands.insert_resource(NextState(GameState::Playing));
    }
}

fn generate_recipe_bbcode(inventory: &PlayerInventory, item: &Item) -> String {
    let mut ingredients = item
        .ingredients()
        .into_iter()
        .fold(HashMap::<Part, u32>::new(), |mut acc, part| {
            *acc.entry(part).or_default() += 1;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();
    ingredients.sort_by(|(part_a, _), (part_b, _)| part_a.cmp(part_b));

    let mut recipe_bbcode = String::new();
    for (part, count) in ingredients {
        let player_count = inventory
            .get_parts()
            .get(&part)
            .copied()
            .unwrap_or_default();

        let line = format!("{:?}: ({}/{})", part, player_count, count);

        if player_count >= count {
            writeln!(&mut recipe_bbcode, "[color=green]{}[/color]", line).unwrap();
        } else {
            writeln!(&mut recipe_bbcode, "[color=red]{}[/color]", line).unwrap();
        }
    }

    recipe_bbcode
}

fn update_recipe_preview(
    recipe: Query<&CraftingTarget>,
    player_inventory: Query<&PlayerInventory>,
    target_changed: Query<(), Changed<CraftingTarget>>,
    player_inv_changed: Query<(), Changed<PlayerInventory>>,
    mut ui: Query<&mut ShelterUi>,
    crafting_assets: Res<CraftingAssets>,
    assets: Res<Assets<GodotResource>>,
) {
    if target_changed.get_single().is_err() && player_inv_changed.get_single().is_err() {
        return;
    }

    let player_inventory = player_inventory.single();
    let mut ui = ui.single_mut();
    let target = recipe.single().0;

    ui.crafting_recipe_text
        .get::<RichTextLabel>()
        .set_bbcode(generate_recipe_bbcode(player_inventory, &target));

    let texture_handle = target.as_texture_handle(&crafting_assets);
    let texture = assets
        .get(texture_handle)
        .unwrap()
        .0
        .clone()
        .cast::<Texture>()
        .unwrap();

    ui.crafting_recipe_preview
        .get::<TextureRect>()
        .set_texture(texture);
}
