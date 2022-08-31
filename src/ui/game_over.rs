use crate::prelude::*;
use bevy_godot::prelude::godot_prelude::Control;

use crate::GameState;

pub struct GameOverUiPlugin;

impl Plugin for GameOverUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(connect_game_over_button)
            .add_startup_system(label_game_over_screen)
            .add_system(listen_to_restart_button.run_in_state(GameState::GameOver))
            .add_system(listen_to_restart_action.run_in_state(GameState::GameOver))
            .add_enter_system(GameState::GameOver, show_game_over_screen)
            .add_enter_system(GameState::Playing, hide_game_over_screen);
    }
}

fn connect_game_over_button(
    mut scene_tree_ref: SceneTreeRef,
    mut entities: Query<(&Name, &mut ErasedGodotRef)>,
) {
    let mut button = entities
        .iter_mut()
        .find_entity_by_name("RestartButton")
        .unwrap();
    connect_godot_signal(&mut button, "pressed", &mut scene_tree_ref);
}

fn listen_to_restart_button(mut events: EventReader<GodotSignal>, mut commands: Commands) {
    for event in events.iter() {
        if event.name() == "pressed" {
            commands.insert_resource(NextState(GameState::Playing));
        }
    }
}

fn listen_to_restart_action(mut commands: Commands) {
    let input = Input::godot_singleton();

    if input.is_action_just_pressed("ui_accept", false) {
        commands.insert_resource(NextState(GameState::Playing));
    }
}

#[derive(Component)]
struct GameOverScreen;

fn label_game_over_screen(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let screen = entities
        .iter()
        .find_entity_by_name("GameOverScreen")
        .unwrap();

    commands.entity(screen).insert(GameOverScreen);
}

fn show_game_over_screen(mut screen: Query<&mut ErasedGodotRef, With<GameOverScreen>>) {
    debug!("Showing game over.");
    let mut screen = screen.single_mut();
    screen.get::<Control>().set_visible(true);
}

fn hide_game_over_screen(mut screen: Query<&mut ErasedGodotRef, With<GameOverScreen>>) {
    debug!("Hiding game over.");
    let mut screen = screen.single_mut();
    screen.get::<Control>().set_visible(false);
}
