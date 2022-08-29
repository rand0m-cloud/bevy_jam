use bevy_godot::prelude::{bevy_prelude::EventReader, *};
use std::time::{Duration, Instant};

pub struct ItemLogPlugin;
impl Plugin for ItemLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(label_item_log)
            .add_system(process_item_log)
            .add_system(visual_update_item_log.as_visual_system())
            .add_event::<ItemLogEvent>();
    }
}

#[derive(Component, Default)]
pub struct ItemLogText {
    entries: Vec<(ItemLogEvent, Instant)>,
    text: String,
}

impl ItemLogText {
    fn add_events<'a>(&mut self, events: impl IntoIterator<Item = &'a ItemLogEvent>) {
        let now = Instant::now();
        self.entries
            .extend(events.into_iter().cloned().map(|log| (log, now)));
    }
    fn update(&mut self) {
        self.entries
            .retain(|entry| entry.1.elapsed() < Duration::from_secs_f32(4.0));

        self.text = self
            .entries
            .iter()
            .fold(String::new(), |mut acc, (event, _)| {
                acc += &event.0;
                acc += "\n";
                acc
            })
    }

    fn get_text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone)]
pub struct ItemLogEvent(pub String);

fn label_item_log(mut commands: Commands, entities: Query<(&Name, Entity)>) {
    let ent = entities
        .iter()
        .find_map(|(name, ent)| (name.as_str() == "ItemPickupText").then_some(ent))
        .unwrap();

    commands.entity(ent).insert(ItemLogText::default());
}

fn process_item_log(mut text: Query<&mut ItemLogText>, mut log_events: EventReader<ItemLogEvent>) {
    let mut log = text.single_mut();

    log.add_events(log_events.iter());
    log.update();
}

fn visual_update_item_log(mut text: Query<(&ItemLogText, &mut ErasedGodotRef)>) {
    let (log, mut reference) = text.single_mut();
    reference.get::<Label>().set_text(log.get_text());
}
