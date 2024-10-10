use bevy_time::{Timer, TimerMode};
use valence::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct DespawnTimer(pub Timer);

impl DespawnTimer {
    pub fn from_secs(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

pub struct DespawnTimerPlugin;

impl Plugin for DespawnTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_entities);
    }
}

fn despawn_entities(
    mut commands: Commands,
    time: Res<bevy_time::Time>,
    mut entities: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut timer) in entities.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).insert(Despawned);
        }
    }
}
