use crate::player::prelude::PlayerStamina;
use crate::prelude::{
    godot_prelude::{AudioStream, AudioStreamPlayer},
    *,
};

#[derive(AssetCollection)]
pub struct PlayerAudioAssets {
    #[asset(path = "art/fatigued-breath.wav.res")]
    fatigued_breath: Handle<GodotResource>,

    #[asset(path = "art/intensive-breath.wav.res")]
    intense_breath: Handle<GodotResource>,

    #[asset(path = "art/out-of-breath.wav.res")]
    exhausted_breath: Handle<GodotResource>,
}

pub struct PlayerAudioPlugin;
impl Plugin for PlayerAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_breath_player)
            .add_system(
                play_breathing
                    .as_visual_system()
                    .run_in_state(GameState::Playing),
            )
            .add_enter_system(GameState::GameOver, game_over);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CurrentBreathingType {
    Fatigued,
    Intense,
    Exhausted,
}

#[derive(Component)]
pub struct PlayerBreathPlayer(Option<CurrentBreathingType>);

fn setup_breath_player(mut commands: Commands, mut scene_tree: SceneTreeRef) {
    let mut player = unsafe { ErasedGodotRef::new(AudioStreamPlayer::new()) };
    scene_tree.add_to_root(player.get::<Node>());

    player.get::<AudioStreamPlayer>().set_bus("BreathingAudio");

    commands
        .spawn()
        .insert(player)
        .insert(PlayerBreathPlayer(None));
}

fn play_breathing(
    mut breath_player: Query<(&mut ErasedGodotRef, &mut PlayerBreathPlayer)>,
    stamina: Query<&PlayerStamina>,
    audio_assets: Res<PlayerAudioAssets>,
    mut assets: ResMut<Assets<GodotResource>>,
) {
    let (mut breath_player, mut player_breath) = breath_player.single_mut();
    let stamina = stamina.single();

    let breath_player = breath_player.get::<AudioStreamPlayer>();

    let breath_audio = if stamina.0 < 0.2 {
        Some(CurrentBreathingType::Exhausted)
    } else if stamina.0 < 0.5 {
        Some(CurrentBreathingType::Intense)
    } else if stamina.0 < 0.85 {
        Some(CurrentBreathingType::Fatigued)
    } else {
        None
    };

    if breath_audio != player_breath.0 {
        if breath_player.is_playing() {
            if breath_player.get_playback_position() < 0.5 {
                return;
            }

            breath_player.stop();
        }

        player_breath.0 = breath_audio;

        if let Some(breath_audio) = breath_audio {
            let audio = match breath_audio {
                CurrentBreathingType::Fatigued => &audio_assets.fatigued_breath,
                CurrentBreathingType::Intense => &audio_assets.intense_breath,
                CurrentBreathingType::Exhausted => &audio_assets.exhausted_breath,
            };
            breath_player.set_stream(assets.get_mut(audio).unwrap().get::<AudioStream>());

            breath_player.play(0.0);
        }
    }
}

fn game_over(mut breath_player: Query<&mut ErasedGodotRef, With<PlayerBreathPlayer>>) {
    let mut breath_player = breath_player.single_mut();
    breath_player.get::<AudioStreamPlayer>().stop();
}
