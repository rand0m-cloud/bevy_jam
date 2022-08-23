use bevy::log::*;
use bevy_godot::prelude::godot_prelude::Color;
use bevy_godot::prelude::{bevy_prelude::With, *};
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct ShelterUiPlugin;

impl Plugin for ShelterUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_shelter_ui)
            .add_system(debug_toggle_shelter_mode.as_visual_system())
            .add_enter_system(GameState::Sheltered, show_shelter_ui)
            .add_enter_system(GameState::Playing, hide_shelter_ui);
    }
}

#[derive(Component)]
struct ShelterUi;

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

    let mut craft_button = entities
        .iter_mut()
        .find_map(|(name, _, reference)| (name.as_str() == "CraftButton").then_some(reference))
        .unwrap();

    craft_button.get::<Button>().set_disabled(true);

    connect_godot_signal(&mut craft_button, "pressed", &mut scene_tree);

    // the hbox container that holds the craftable options
    let mut craftables = entities
        .iter_mut()
        .find_map(|(name, _, reference)| (name.as_str() == "Craftables").then_some(reference))
        .unwrap();

    // mark craftables as uncraftable
    for obj in craftables.get::<Node>().get_children().into_iter() {
        let obj = unsafe { obj.to_object::<Control>().unwrap().assume_safe() };
        obj.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.3));
    }
}

fn show_shelter_ui(mut screen: Query<&mut ErasedGodotRef, With<ShelterUi>>) {
    debug!("Showing shelter ui.");
    let mut screen = screen.single_mut();
    let screen = screen.get::<Control>();
    screen.set_visible(true);
}

fn hide_shelter_ui(mut screen: Query<&mut ErasedGodotRef, With<ShelterUi>>) {
    debug!("Hiding shelter ui.");
    let mut screen = screen.single_mut();
    let screen = screen.get::<Control>();
    screen.set_visible(false);
}

fn debug_toggle_shelter_mode(mut commands: Commands, state: Res<CurrentState<GameState>>) {
    let input = Input::godot_singleton();

    if input.is_action_just_pressed("debug_enter_shelter_mode", false) {
        if state.0 == GameState::Playing {
            commands.insert_resource(NextState(GameState::Sheltered));
        } else {
            commands.insert_resource(NextState(GameState::Playing));
        }
    }
}
