use crate::{crafting::Item, player::Player};
use bevy::log::*;
use bevy_godot::prelude::{
    bevy_prelude::{EventReader, ParamSet, With, Without},
    godot_prelude::Color,
    *,
};
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct ShelterUiPlugin;

impl Plugin for ShelterUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_shelter_ui)
            .add_system(debug_toggle_shelter_mode.as_visual_system())
            .add_system(listen_for_crafting_ui_presses.run_in_state(GameState::Sheltered))
            .add_enter_system(GameState::Sheltered, show_shelter_ui)
            .add_enter_system(GameState::Playing, hide_shelter_ui);
    }
}

#[derive(Component)]
struct ShelterUi;

#[derive(Component)]
struct CraftingUi;

#[derive(Component)]
struct CraftButton;

#[derive(Component)]
struct CraftingTarget(Option<Item>);

#[derive(Component)]
struct CraftingTargetText;

fn setup_shelter_ui(
    mut commands: Commands,
    mut entities: Query<(&Name, Entity, &mut ErasedGodotRef)>,
    mut scene_tree: SceneTreeRef,
) {
    let screen = entities
        .iter_mut()
        .find_map(|(name, ent, _)| (name.as_str() == "ShelterUI").then_some(ent))
        .unwrap();

    commands.entity(screen).insert(ShelterUi);

    // setup craft button
    let (craft_button_ent, mut craft_button) = entities
        .iter_mut()
        .find_map(|(name, ent, reference)| {
            (name.as_str() == "CraftButton").then_some((ent, reference))
        })
        .unwrap();

    craft_button.get::<Button>().set_disabled(true);

    connect_godot_signal(&mut craft_button, "pressed", &mut scene_tree);

    commands.entity(craft_button_ent).insert(CraftButton);

    // the hbox container that holds the craftable options
    let mut craftables = entities
        .iter_mut()
        .find_map(|(name, _, reference)| (name.as_str() == "Craftables").then_some(reference))
        .unwrap();

    // setup item nodes in the crafting menu
    for obj in craftables.get::<Node>().get_children().into_iter() {
        let mut obj =
            unsafe { ErasedGodotRef::new(obj.to_object::<Control>().unwrap().assume_unique()) };

        let item = Item::from_str(&obj.get::<Node>().name().to_string())
            .expect("to get Item from node's name");

        // setup button pressed signals
        connect_godot_signal(&mut obj, "pressed", &mut scene_tree);

        // insert component
        let obj_ent = entities
            .iter_mut()
            .find_map(|(_, ent, reference)| {
                (reference.instance_id() == obj.instance_id()).then_some(ent)
            })
            .unwrap();

        commands.entity(obj_ent).insert(CraftingUi).insert(item);
    }

    // setup crafting target preview node
    let craft_target = entities
        .iter()
        .find_map(|(name, ent, _)| (name.as_str() == "SelectedItem").then_some(ent))
        .unwrap();

    commands
        .entity(craft_target)
        .insert(CraftingUi)
        .insert(CraftingTarget(None));

    // setup crafting target text
    let craft_target_text = entities
        .iter()
        .find_map(|(name, ent, _)| (name.as_str() == "SelectedItemText").then_some(ent))
        .unwrap();

    commands
        .entity(craft_target_text)
        .insert(CraftingUi)
        .insert(CraftingTargetText);
}

fn refresh_crafting_ui(
    player: &Player,
    items: &mut Query<(&Item, &mut ErasedGodotRef), With<CraftingUi>>,
) {
    for (item, mut reference) in items.iter_mut() {
        let reference = reference.get::<Control>();

        // mark items as craftable
        if player.inventory.can_craft(*item) {
            reference.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 1.0));
        } else {
            reference.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.3));
        }
    }
}

fn show_shelter_ui(
    mut screen: Query<&mut ErasedGodotRef, (With<ShelterUi>, Without<CraftingUi>)>,
    player: Query<&Player>,
    mut items: Query<(&Item, &mut ErasedGodotRef), With<CraftingUi>>,
) {
    debug!("Showing shelter ui.");
    let mut screen = screen.single_mut();
    let screen = screen.get::<Control>();
    screen.set_visible(true);

    // refresh available crafting options
    refresh_crafting_ui(player.single(), &mut items);
}

fn hide_shelter_ui(mut screen: Query<&mut ErasedGodotRef, With<ShelterUi>>) {
    debug!("Hiding shelter ui.");
    let mut screen = screen.single_mut();
    let screen = screen.get::<Control>();
    screen.set_visible(false);
}

fn listen_for_crafting_ui_presses(
    mut events: EventReader<GodotSignal>,
    mut player: Query<&mut Player>,
    mut crafting_target: Query<&mut CraftingTarget>,
    mut queries: ParamSet<(
        Query<&mut ErasedGodotRef, With<CraftButton>>,
        Query<(&Item, &mut ErasedGodotRef), With<CraftingUi>>,
        Query<&mut ErasedGodotRef, With<CraftingTargetText>>,
    )>,
) {
    let mut player = player.single_mut();

    for event in events.iter() {
        let mut craft_button = queries.p0();
        let mut craft_button = craft_button.single_mut();
        let craft_button = craft_button.get::<Button>();

        if event.name() == "pressed" {
            let node_name = event.origin().get::<Node>().name().to_string();

            if let Some(item) = Item::from_str(&node_name) {
                let mut crafting_target = crafting_target.single_mut();
                crafting_target.0 = Some(item);

                if player.inventory.can_craft(item) {
                    craft_button.set_disabled(false);
                } else {
                    craft_button.set_disabled(true);
                }

                // set the craft target text
                let mut craft_target_text = queries.p2();
                let mut craft_target_text = craft_target_text.single_mut();
                craft_target_text.get::<Label>().set_text(item.as_str());
            } else if node_name == "CraftButton" {
                if let Some(target) = crafting_target.single_mut().0 {
                    debug!("trying to craft: {:?}", target);

                    player.inventory.craft(target);

                    if player.inventory.can_craft(target) {
                        craft_button.set_disabled(false);
                    } else {
                        craft_button.set_disabled(true);
                    }

                    refresh_crafting_ui(&player, &mut queries.p1());
                }
            }
        }
    }
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
}
