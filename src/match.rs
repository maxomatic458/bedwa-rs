use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    query::With,
    system::{Commands, Query, Res, ResMut},
    world::OnRemove,
};
use bevy_state::{
    prelude::in_state,
    state::{NextState, OnEnter},
};
use bevy_time::{Time, Timer, TimerMode};
use valence::{
    app::{Plugin, Update},
    client::{Client, Username},
    entity::{item::Stack, Position},
    equipment::EquipmentInventorySync,
    math::DVec3,
    message::SendMessage,
    player_list::DisplayName,
    prelude::{Block, DetectChanges, Equipment, IntoSystemConfigs, Inventory, Resource, Trigger},
    protocol::{sound::SoundCategory, Sound},
    title::SetTitle,
    BlockPos, BlockState, ChunkLayer, Despawned, GameMode, ItemKind, ItemStack,
};

use crate::{
    base::{
        break_blocks::BedDestroyedEvent,
        build::PlayerPlacedBlocks,
        combat::CombatState,
        death::{IsDead, PlayerDeathEvent},
        fall_damage::FallingState,
        physics::CollidableForEntities,
        scoreboard::BedwarsScoreboard,
    },
    bedwars_config::WorldConfig,
    utils::inventory::InventoryExt,
    GameState, LobbyPlayer, Spectator, Team,
};

/// Time to wait before sending all the players back to the lobby after the match ends.
pub const POST_MATCH_TIME_SECS: f32 = 10.0;

#[derive(Debug, Clone, Resource)]
pub struct MatchState {
    pub started: std::time::Instant,
    pub player_stats: HashMap<String, PlayerStats>,
    pub teams: HashMap<String, TeamState>,
}

impl Default for MatchState {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchState {
    pub fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
            player_stats: HashMap::new(),
            teams: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerStats {
    pub deaths: u16,
    pub kills: u16,
    pub beds_destroyed: u16,
    pub resources_collected: HashMap<ItemKind, u64>,
    pub resources_spent: HashMap<ItemKind, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct TeamState {
    pub players: Vec<String>,
    pub players_alive: Vec<String>,
    pub bed_destroyed: bool,
}

#[derive(Debug, Clone, Resource)]
struct PostMatchTimer(pub Timer);

impl Default for PostMatchTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(POST_MATCH_TIME_SECS, TimerMode::Once))
    }
}

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut valence::app::App) {
        app.add_systems(OnEnter(GameState::Match), (start_match,))
            .add_systems(
                Update,
                (on_bed_destroy, on_player_death).run_if(in_state(GameState::Match)),
            )
            .add_event::<EndMatch>()
            .add_systems(Update, (on_end_match,).run_if(in_state(GameState::Match)))
            .add_systems(OnEnter(GameState::PostMatch), (on_enter_post_match,))
            .add_systems(
                Update,
                (tick_postmatch_timer,).run_if(in_state(GameState::PostMatch)),
            )
            .observe(on_remove_team);
    }
}

fn start_match(
    mut commands: Commands,
    mut players: Query<(
        Entity,
        &mut Position,
        &mut Inventory,
        &mut Client,
        &mut GameMode,
        &Username,
        &Team,
    )>,
    bedwars_config: Res<WorldConfig>,
) {
    tracing::info!("Starting match");

    let mut match_state = MatchState::new();
    for team in &bedwars_config.teams {
        match_state
            .teams
            .insert(team.0.to_string(), TeamState::default());
    }

    for (entity, mut pos, mut inventory, _client, mut game_mode, username, team) in
        players.iter_mut()
    {
        *game_mode = GameMode::Survival;
        inventory.clear();
        inventory.readonly = false;

        inventory.set_slot(36, ItemStack::new(ItemKind::Brick, 64, None));
        inventory.set_slot(37, ItemStack::new(ItemKind::IronIngot, 64, None));
        inventory.set_slot(38, ItemStack::new(ItemKind::GoldIngot, 64, None));

        let team_spawn = bedwars_config.spawns.get(&team.name).unwrap();
        pos.set(team_spawn.clone());

        commands
            .entity(entity)
            .insert(CombatState::default())
            .insert(FallingState::default())
            .insert(Equipment::default())
            .insert(CollidableForEntities)
            .insert(EquipmentInventorySync);

        match_state
            .player_stats
            .insert(username.0.clone(), PlayerStats::default());

        let team = match_state.teams.get_mut(&team.name).unwrap();

        team.players_alive.push(username.to_string());
        team.players.push(username.to_string());
    }

    commands.insert_resource(match_state);
}

fn on_bed_destroy(
    mut clients: Query<(&mut Client, &Team)>,
    mut events: EventReader<BedDestroyedEvent>,
    mut layer: Query<&mut ChunkLayer>,
    mut match_state: ResMut<MatchState>,
    bedwars_config: Res<WorldConfig>,
) {
    for event in events.read() {
        for (mut client, team) in &mut clients {
            if team == &event.team {
                client.set_title("§cyour bed was destroyed!");
                client.send_chat_message("§cYour bed was destroyed!");
            }
        }

        let bed_pos = bedwars_config.beds.get(&event.team.name).unwrap();

        match_state
            .teams
            .get_mut(&event.team.name)
            .unwrap()
            .bed_destroyed = true;

        let mut layer = layer.single_mut();
        layer.play_sound(
            Sound::EntityHorseDeath,
            SoundCategory::Master,
            Into::<DVec3>::into(bed_pos[0].0.clone()),
            1.0,
            1.0,
        );
    }
}

fn on_player_death(
    mut events: EventReader<PlayerDeathEvent>,
    players: Query<(&Username, &Team)>,
    mut match_state: ResMut<MatchState>,
) {
    for event in events.read() {
        let Ok((victim_name, victim_team)) = players.get(event.victim) else {
            continue;
        };

        let victim_team_state = match_state.teams.get_mut(&victim_team.name).unwrap();
        if victim_team_state.bed_destroyed {
            victim_team_state
                .players_alive
                .retain(|p| p != &victim_name.0);
        }

        let victim_stats = match_state.player_stats.get_mut(&victim_name.0).unwrap();
        victim_stats.deaths += 1;

        if let Some(attacker) = event.attacker {
            let Ok((attacker_name, _)) = players.get(attacker) else {
                continue;
            };

            let attacker_stats = match_state.player_stats.get_mut(&attacker_name.0).unwrap();
            attacker_stats.kills += 1;
        }
    }
}

#[derive(Debug, Clone, Event)]
pub struct EndMatch {
    pub winner: Team,
}

fn on_end_match(
    // commands: Commands,
    match_state: ResMut<MatchState>,
    bedwars_config: Res<WorldConfig>,
    mut clients: Query<(Entity, &mut Client, &Position, &Team), With<Client>>,
    mut event_writer: EventWriter<EndMatch>,
    mut state: ResMut<NextState<GameState>>,
) {
    if !match_state.is_changed() {
        return;
    }

    let teams_left = match_state
        .teams
        .iter()
        .filter(|(_, team)| !team.players_alive.is_empty())
        .count();

    // DEBUG
    if teams_left >= 1 {
        return;
    }

    let winner = match_state
        .teams
        .iter()
        .find(|(_, team)| !team.players_alive.is_empty())
        .map(|(name, _)| bedwars_config.teams.get_key_value(name).unwrap())
        .unwrap();

    event_writer.send(EndMatch {
        winner: Team {
            name: winner.0.to_string(),
            color: *winner.1,
        },
    });

    for (_ent, mut client, position, team) in &mut clients {
        if team.name == *winner.0 {
            client.play_sound(
                Sound::EntityPlayerLevelup,
                SoundCategory::Player,
                position.0,
                0.75,
                1.0,
            );
        }
    }

    state.set(GameState::PostMatch);
}

fn on_enter_post_match(mut commands: Commands) {
    commands.insert_resource(PostMatchTimer::default());
}

#[allow(clippy::too_many_arguments)]
fn tick_postmatch_timer(
    mut commands: Commands,
    mut players: Query<(Entity, &Client)>,
    mut timer: ResMut<PostMatchTimer>,
    mut state: ResMut<NextState<GameState>>,
    scoreboard: Query<Entity, With<BedwarsScoreboard>>,
    items: Query<Entity, With<Stack>>,
    player_placed_blocks: ResMut<PlayerPlacedBlocks>,
    mut layer: Query<&mut ChunkLayer>,
    time: Res<Time>,
    bedwars_config: Res<WorldConfig>,
) {
    timer.0.tick(time.delta());

    if !timer.0.finished() {
        return;
    }
    for (ent, _client) in &mut players {
        commands
            .entity(ent)
            .remove::<CombatState>()
            .remove::<FallingState>()
            .remove::<Equipment>()
            .remove::<CollidableForEntities>()
            .remove::<EquipmentInventorySync>()
            .remove::<Team>()
            .remove::<IsDead>()
            .remove::<Spectator>()
            .insert(LobbyPlayer);
    }

    for scoreboard in &mut scoreboard.iter() {
        commands.entity(scoreboard).insert(Despawned);
    }

    let mut layer = layer.single_mut();

    for block_pos in player_placed_blocks.0.iter() {
        layer.set_block(*block_pos.0, BlockState::AIR);
    }

    // Despawn items
    for item in &mut items.iter() {
        commands.entity(item).insert(Despawned);
    }

    // Replace beds
    for (_, bed_blocks_map) in bedwars_config.beds.iter() {
        for (pos, block) in bed_blocks_map.iter() {
            let block_pos = BlockPos::new(pos.x, pos.y, pos.z);
            layer.set_block(block_pos, Block::from(block.clone()));
        }
    }

    state.set(GameState::Lobby);
}

fn on_remove_team(trigger: Trigger<OnRemove, Team>, mut clients: Query<&mut DisplayName>) {
    let Ok(mut display_name) = clients.get_mut(trigger.entity()) else {
        return;
    };

    display_name.0 = None;
}
