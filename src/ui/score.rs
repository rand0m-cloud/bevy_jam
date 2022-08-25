use crate::{GameState, Score};
use bevy_godot::prelude::{bevy_prelude::With, *};
use iyes_loopless::prelude::*;

pub struct ScoreUiPlugin;
impl Plugin for ScoreUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreTimer(Timer::from_seconds(2.5, true)))
            .add_startup_system(label_score_ui)
            .add_system(update_score_ui.as_visual_system())
            .add_system(update_score_timer.as_visual_system())
            .add_exit_system(GameState::GameOver, reset_score);
    }
}

#[derive(Component, Debug)]
pub struct ScoreLabel;

pub struct ScoreTimer(Timer);

fn label_score_ui(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let score_ui_ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "ScoreLabel").then_some(ent))
        .unwrap();

    commands.entity(score_ui_ent).insert(ScoreLabel);
}

fn update_score_ui(score: Res<Score>, mut score_ui: Query<&mut ErasedGodotRef, With<ScoreLabel>>) {
    if score.is_changed() {
        let mut score_ui = score_ui.single_mut();
        score_ui
            .get::<Label>()
            .set_text(format!("Score: {}", score.0));
    }
}

fn update_score_timer(
    mut score: ResMut<Score>,
    mut time: SystemDelta,
    mut score_timer: ResMut<ScoreTimer>,
) {
    let delta = time.delta();

    score_timer.0.tick(delta);
    if score_timer.0.just_finished() {
        score.0 += 50;
    }
}

fn reset_score(mut score: ResMut<Score>, mut score_timer: ResMut<ScoreTimer>) {
    score.0 = 0;
    score_timer.0.reset();
}
