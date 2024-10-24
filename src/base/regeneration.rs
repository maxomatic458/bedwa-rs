use bevy_ecs::query::QueryData;
use bevy_state::prelude::in_state;
use bevy_time::{Time, Timer, TimerMode};
use valence::{entity::living::Health, prelude::*};

use crate::GameState;

use super::combat::CombatState;

pub struct RegenerationPlugin;

const SECS_PER_HP: f32 = 4.0;
// const COMBAT_COOLDOWN_TICKS: u128 = 140;
const COMBAT_COOLDOWN_MILLIS: u128 = 7000;

#[derive(Component)]
struct RegenTimer(Timer);

impl Default for RegenTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SECS_PER_HP, TimerMode::Repeating))
    }
}

impl Plugin for RegenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EventLoopUpdate,
            regeneration_system.run_if(in_state(GameState::Match)),
        );
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct RegenerationQuery {
    entity: Entity,
    health: &'static mut Health,
    combat_state: &'static CombatState,
}

fn regeneration_system(
    mut commands: Commands,
    mut query: Query<(RegenerationQuery, Option<&mut RegenTimer>)>,
    time: Res<Time>,
) {
    for (mut query, timer) in query.iter_mut() {
        if query.combat_state.last_hit.elapsed().as_millis() < COMBAT_COOLDOWN_MILLIS {
            // player entered / is in combat, remove the regen timer
            commands.entity(query.entity).remove::<RegenTimer>();
        } else if let Some(mut timer) = timer {
            if timer.0.tick(time.delta()).just_finished() {
                let new_health = (query.health.0 + 1.0).min(20.0);
                query.health.0 = new_health;
            }
        } else {
            // player is not in combat, add the regen timer
            commands.entity(query.entity).insert(RegenTimer::default());
        }
    }
}
