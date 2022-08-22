use bevy::prelude::*;
use bevy_godot::prelude::*;
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(connect_game_over_button)
            .add_startup_system(label_game_over_screen)
            .add_system(listen_to_restart_button)
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
        .find_map(|(name, ent)| (name.as_str() == "RestartButton").then_some(ent))
        .unwrap();
    connect_godot_signal(&mut button, "pressed", &mut scene_tree_ref);
}

fn listen_to_restart_button(mut events: EventReader<GodotSignal>, mut commands: Commands) {
    for event in events.iter() {
        if event.name() == "pressed" {
            info!("Restart button pressed");
            commands.insert_resource(NextState(GameState::Playing));
        }
    }
}

#[derive(Component)]
struct GameOverScreen;

fn label_game_over_screen(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let screen = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "GameOverScreen").then_some(ent))
        .unwrap();

    commands.entity(screen).insert(GameOverScreen);
}

fn show_game_over_screen(mut screen: Query<&mut ErasedGodotRef, With<GameOverScreen>>) {
    let mut screen = screen.single_mut();
    let screen = screen.get::<Control>();
    screen.set_visible(true)
}

fn hide_game_over_screen(mut screen: Query<&mut ErasedGodotRef, With<GameOverScreen>>) {
    let mut screen = screen.single_mut();
    let screen = screen.get::<Control>();
    screen.set_visible(false)
}
