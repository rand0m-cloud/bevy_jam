use bevy_godot::prelude::{
    bevy_prelude::{Added, Without},
    *,
};

pub struct AlarmPlugin;
impl Plugin for AlarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(label_alarms).add_system(process_alarms);
    }
}

#[derive(Debug, Component)]
pub struct Alarm {
    lifetime_timer: Timer,
    active_period: Timer,
    inactive_period: Timer,
    is_active: bool,
}

impl Default for Alarm {
    fn default() -> Self {
        Self {
            lifetime_timer: Timer::from_seconds(60.0, false),
            active_period: Timer::from_seconds(15.0, true),
            inactive_period: Timer::from_seconds(5.0, true),
            is_active: true,
        }
    }
}

impl Alarm {
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[derive(Debug, Component)]
pub struct AlarmAudioPlayer(Entity);

fn label_alarms(
    mut commands: Commands,
    mut entities: Query<(&Groups, Entity, &mut ErasedGodotRef), Added<ErasedGodotRef>>,
) {
    let mut alarms = vec![];
    for (groups, ent, mut reference) in entities.iter_mut() {
        if groups.is("alarm") {
            commands.entity(ent).insert(Alarm::default());

            let audio_player_instance_id = unsafe {
                reference
                    .get::<Node2D>()
                    .get_node("AudioStreamPlayer2D")
                    .unwrap()
                    .assume_safe()
                    .get_instance_id()
            };
            alarms.push((ent, audio_player_instance_id));
        }
    }

    for (alarm_ent, audio_player_instance_id) in alarms {
        let audio_player = entities
            .iter()
            .find_map(|(_, ent, reference)| {
                (reference.instance_id() == audio_player_instance_id).then_some(ent)
            })
            .unwrap();

        commands
            .entity(audio_player)
            .insert(AlarmAudioPlayer(alarm_ent));
    }
}

fn process_alarms(
    mut alarms: Query<(&mut Alarm, &mut ErasedGodotRef, Entity)>,
    mut alarm_sfx_players: Query<(&AlarmAudioPlayer, &mut ErasedGodotRef), Without<Alarm>>,
    mut time: SystemDelta,
) {
    let delta = time.delta();

    for (mut alarm, mut reference, alarm_ent) in alarms.iter_mut() {
        let reference = reference.get::<Node2D>();

        let mut sound = alarm_sfx_players
            .iter_mut()
            .find_map(|(audio, reference)| (audio.0 == alarm_ent).then_some(reference))
            .unwrap();
        let sound = sound.get::<AudioStreamPlayer2D>();

        alarm.lifetime_timer.tick(delta);
        if alarm.lifetime_timer.finished() {
            reference.queue_free();
            return;
        }

        if alarm.is_active {
            alarm.active_period.tick(delta);
            if alarm.active_period.just_finished() {
                alarm.inactive_period.reset();
                alarm.is_active = false;
                sound.stop();
            }
        } else {
            alarm.inactive_period.tick(delta);
            if alarm.inactive_period.just_finished() {
                alarm.active_period.reset();
                alarm.is_active = true;
                sound.play(0.0);
            }
        }
    }
}
