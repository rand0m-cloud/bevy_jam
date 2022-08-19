use bevy_godot::prelude::*;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_startup_system(spawn_simple_scene);
}

bevy_godot_init!(init, build_app);

fn spawn_simple_scene(
    mut commands: Commands,
    entities: Query<(&Name, &Transform)>,
) {
    let spawn_location = entities
        .iter()
        .find_map(|(name, transform)| (name.as_str() == "SpawnPosition").then_some(*transform))
        .unwrap();

    commands
        .spawn()
        .insert(spawn_location)
        .insert(GodotScene::from_path("res://simple_scene.tscn"));
}
