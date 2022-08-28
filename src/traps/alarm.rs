use bevy_godot::prelude::{bevy_prelude::Added, *};

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

fn label_alarms(mut commands: Commands, entities: Query<(&Groups, Entity), Added<ErasedGodotRef>>) {
    for (groups, ent) in entities.iter() {
        if groups.is("alarm") {
            commands.entity(ent).insert(Alarm::default());
        }
    }
}

fn process_alarms(mut alarms: Query<(&mut Alarm, &mut ErasedGodotRef)>, mut time: SystemDelta) {
    let delta = time.delta();

    for (mut alarm, mut reference) in alarms.iter_mut() {
        let reference = reference.get::<Node2D>();

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
            }
        } else {
            alarm.inactive_period.tick(delta);
            if alarm.inactive_period.just_finished() {
                alarm.active_period.reset();
                alarm.is_active = true;
            }
        }
    }
}
