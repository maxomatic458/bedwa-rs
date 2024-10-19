use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Query, Res, ResMut},
};
use bevy_state::{prelude::in_state, state::OnEnter};
use valence::{
    app::{Plugin, Update},
    client::{Client, Username},
    entity::Position,
    equipment::EquipmentInventorySync,
    math::DVec3,
    message::SendMessage,
    prelude::{Equipment, IntoSystemConfigs, Inventory, Resource},
    protocol::{sound::SoundCategory, Sound},
    title::SetTitle,
    ChunkLayer, GameMode, ItemKind,
};

use crate::{
    base::{
        break_blocks::BedDestroyedEvent, combat::CombatState, death::PlayerDeathEvent,
        fall_damage::FallingState,
    },
    bedwars_config::BedwarsConfig,
    utils::inventory::InventoryExt,
    GameState, Team,
};

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

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut valence::app::App) {
        app.add_systems(OnEnter(GameState::Match), (start_match,))
            .add_systems(
                Update,
                (on_bed_destroy, on_player_death).run_if(in_state(GameState::Match)),
            );
        // app.add_systems(Update, (

        // ))
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
    bedwars_config: Res<BedwarsConfig>,
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
        inventory.readonly = true;

        let team_spawn = bedwars_config.spawns.get(&team.name).unwrap();
        pos.set(team_spawn.clone());

        commands
            .entity(entity)
            .insert(CombatState::default())
            .insert(FallingState::default())
            .insert(Equipment::default())
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
    bedwars_config: Res<BedwarsConfig>,
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
            Into::<DVec3>::into(bed_pos[0].clone()),
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
        victim_team_state
            .players_alive
            .retain(|p| p != &victim_name.0);

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
