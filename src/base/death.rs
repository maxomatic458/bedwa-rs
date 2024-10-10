
use bevy_ecs::query::Added;
use bevy_state::prelude::in_state;
use bevy_time::{Time, Timer, TimerMode};
use valence::prelude::Inventory;
use valence::{entity::living::Health, prelude::*};

use crate::{
    bedwars_config::{BedwarsConfig},
    r#match::MatchState,
    utils::inventory::{InventoryExt},
    GameState, Team,
};

const PLAYER_RESPAWN_TIMER: f32 = 5.0;

#[derive(Debug, Clone, Component)]
pub struct IsDead;

pub struct DeathPlugin;
#[derive(Debug, Clone, Component)]
pub struct RespawnTimer(pub Timer);

impl Default for RespawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(PLAYER_RESPAWN_TIMER, TimerMode::Once))
    }
}

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(
            Update,
            (
                on_death.run_if(in_state(GameState::Match)),
                tick_respawn_timer,
            ),
        )
        .observe(player_respawn);
    }
}

fn tick_respawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut clients: Query<(Entity, &mut Client, &mut RespawnTimer)>,
) {
    for (player, mut client, mut timer) in &mut clients {
        let seconds_left = timer.0.remaining_secs();
        timer.0.tick(time.delta());

        // client.set_title(format!("Respawn in: {}", seconds_left as u32));

        // client.play_sound(Sound::BlockDispenserDispense, SoundCategory::Block, client.p, volume, pitch);

        if timer.0.finished() {
            commands.entity(player).remove::<IsDead>();
            commands.entity(player).remove::<RespawnTimer>(); // is this required?
            client.clear_title();
        }
    }
}

fn player_respawn(
    trigger: Trigger<OnRemove, IsDead>,
    mut clients: Query<(&mut Position, &mut Health, &mut GameMode, &Team)>,
    bedwars_config: Res<BedwarsConfig>,
) {
    let Ok((mut position, mut health, mut game_mode, team)) = clients.get_mut(trigger.entity())
    else {
        return;
    };

    health.0 = 20.0;
    *game_mode = GameMode::Survival;

    let team_spawn_pos = bedwars_config.spawns.get(&team.0).unwrap();
    position.set(DVec3::new(
        team_spawn_pos.x as f64,
        team_spawn_pos.y as f64,
        team_spawn_pos.z as f64,
    ))
}

fn on_death(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Inventory, &mut GameMode, &Team), Added<IsDead>>,
    match_state: Res<MatchState>,
) {
    for (player, mut inventory, mut game_mode, team) in &mut clients {
        let bed_destroyed = match_state.teams.get(&team.0).unwrap().bed_destroyed;
        *game_mode = GameMode::Spectator;
        inventory.clear();
        if !bed_destroyed {
            let player_respawn_timer = RespawnTimer(Timer::from_seconds(5.0, TimerMode::Once));
            commands.entity(player).insert(player_respawn_timer);
        }
    }
}
