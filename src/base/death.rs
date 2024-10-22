use bevy_ecs::query::Added;
use bevy_state::prelude::in_state;
use bevy_time::{Time, Timer, TimerMode};
use valence::client::UpdateClientsSet;
use valence::entity::EntityId;
use valence::prelude::Inventory;
use valence::protocol::packets::play::EntityDamageS2c;
use valence::protocol::sound::SoundCategory;
use valence::protocol::{Sound, WritePacket};
use valence::Layer;
use valence::{entity::living::Health, prelude::*};

use crate::Spectator;
use crate::{
    bedwars_config::BedwarsConfig, r#match::MatchState, utils::inventory::InventoryExt, GameState,
    Team,
};

const PLAYER_RESPAWN_TIMER_SECS: u32 = 5;

#[derive(Debug, Clone, Component)]
pub struct IsDead;

pub struct DeathPlugin;
#[derive(Debug, Clone, Component)]
pub struct RespawnTimer {
    pub repeats: u32,
    pub timer: Timer,
}

impl Default for RespawnTimer {
    fn default() -> Self {
        Self {
            repeats: PLAYER_RESPAWN_TIMER_SECS,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Clone, Event)]
pub struct PlayerDeathEvent {
    pub attacker: Option<Entity>, // <- player killed themselves
    pub victim: Entity,
    pub position: DVec3,
}

#[derive(Debug, Clone, Event)]
pub struct PlayerHurtEvent {
    pub attacker: Option<Entity>, // <- player killed themselves
    pub victim: Entity,
    pub position: DVec3,
    pub damage: f32,
}

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(
            Update,
            (
                on_player_hurt
                    .before(UpdateClientsSet)
                    .run_if(in_state(GameState::Match)),
                on_death.run_if(in_state(GameState::Match)),
                tick_respawn_timer,
            ),
        )
        .add_event::<PlayerHurtEvent>()
        .add_event::<PlayerDeathEvent>()
        .observe(player_respawn);
    }
}

fn tick_respawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut clients: Query<(Entity, &mut Client, &Position, &mut RespawnTimer)>,
) {
    for (player, mut client, position, mut timer) in &mut clients {
        if timer.timer.tick(time.delta()).just_finished() {
            timer.repeats -= 1;
            client.set_title_times(0, 21, 0);
            client.set_title(format!("Respawn in: {}", timer.repeats));

            client.play_sound(
                Sound::BlockDispenserDispense,
                SoundCategory::Master,
                position.0,
                2.0,
                1.0,
            );
        }

        if timer.timer.finished() && timer.repeats == 0 {
            commands.entity(player).remove::<IsDead>();
            commands.entity(player).remove::<RespawnTimer>(); // is this required?
            client.clear_title();
        }
    }
}

fn player_respawn(
    trigger: Trigger<OnRemove, IsDead>,
    mut clients: Query<
        (
            &mut Position,
            &mut Health,
            &mut GameMode,
            &Team,
            &mut Equipment,
        ),
        Without<Spectator>,
    >,
    bedwars_config: Res<BedwarsConfig>,
) {
    let Ok((mut position, mut health, mut game_mode, team, mut equipment)) =
        clients.get_mut(trigger.entity())
    else {
        return;
    };

    health.0 = 20.0;
    *game_mode = GameMode::Survival;

    equipment.set_changed();

    let team_spawn_pos = bedwars_config.spawns.get(&team.name).unwrap();
    position.set(DVec3::new(
        team_spawn_pos.x as f64,
        team_spawn_pos.y as f64,
        team_spawn_pos.z as f64,
    ))
}

#[allow(clippy::type_complexity)]
fn on_death(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &Position,
            &mut Inventory,
            &mut GameMode,
            &Team,
            &mut Health,
        ),
        Added<IsDead>,
    >,
    match_state: Res<MatchState>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for (player_ent, position, mut inventory, mut game_mode, team, mut health) in &mut clients {
        let bed_destroyed = match_state.teams.get(&team.name).unwrap().bed_destroyed;
        *game_mode = GameMode::Spectator;
        inventory.clear();
        health.0 = 20.0;

        let mut layer = layer.single_mut();
        layer.play_sound(
            Sound::EntityPlayerDeath,
            SoundCategory::Master,
            position.0,
            1.0,
            1.0,
        );

        if !bed_destroyed {
            let player_respawn_timer = RespawnTimer::default();
            commands.entity(player_ent).insert(player_respawn_timer);
        } else {
            commands.entity(player_ent).insert(Spectator);
            // commands.entity(player_ent).remove::<IsDead>();
            commands.entity(player_ent).remove::<Team>();
        }
    }
}

fn on_player_hurt(
    mut commands: Commands,
    mut clients: Query<(&EntityId, &mut Health)>,
    mut events: EventReader<PlayerHurtEvent>,
    mut event_writer: EventWriter<PlayerDeathEvent>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for event in events.read() {
        let Ok((victim_id, mut victim_health)) = clients.get_mut(event.victim) else {
            continue;
        };

        let victim_id = victim_id.get();

        let mut layer = layer.single_mut();

        let new_health = victim_health.0 - event.damage;

        if new_health <= 0.0 {
            event_writer.send(PlayerDeathEvent {
                attacker: event.attacker,
                victim: event.victim,
                position: event.position,
            });
            layer.play_sound(
                Sound::EntityPlayerDeath,
                SoundCategory::Player,
                event.position,
                1.0,
                1.0,
            );
            commands.entity(event.victim).insert(IsDead);
        } else {
            layer.play_sound(
                Sound::EntityPlayerHurt,
                SoundCategory::Player,
                event.position,
                1.0,
                1.0,
            );
            victim_health.0 = new_health;
        }

        let attacker_id = event
            .attacker
            .map(|attacker| clients.get(attacker).map(|(id, _)| *id).unwrap_or_default());

        layer
            .view_writer(event.position)
            .write_packet(&EntityDamageS2c {
                entity_id: victim_id.into(),
                source_type_id: 1.into(), // idk what 1 is, probably physical damage
                source_cause_id: attacker_id.unwrap_or_default().get().into(),
                source_direct_id: attacker_id.unwrap_or_default().get().into(),
                source_pos: Some(event.position),
            });
    }
}
