use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res},
};
use bevy_state::state::OnEnter;
use valence::{
    app::Plugin,
    client::{Client, Username},
    entity::Position,
    prelude::{Inventory, Resource},
    GameMode, ItemKind, ItemStack,
};

use crate::{
    base::{combat::CombatState, fall_damage::FallingState},
    bedwars_config::BedwarsConfig,
    utils::inventory::InventoryExt,
    GameState, Team,
};

#[derive(Debug, Clone, Resource)]
pub struct MatchState {
    pub started: std::time::Instant,
    pub player_stats: HashMap<String, PlayerStats>,
    pub teams: HashMap<String, TeamStats>,
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
pub struct TeamStats {
    pub players: Vec<String>,
    pub players_alive: Vec<String>,
    pub bed_destroyed: bool,
}

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut valence::app::App) {
        app.add_systems(OnEnter(GameState::Match), (start_match,));
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
            .insert(team.0.to_string(), TeamStats::default());
    }

    for (entity, mut pos, mut inventory, client, mut game_mode, username, team) in
        players.iter_mut()
    {
        *game_mode = GameMode::Survival;
        inventory.clear();
        inventory.set_slot(9, ItemStack::new(ItemKind::Dirt, 64, None));
        inventory.readonly = false;

        let team_spawn = bedwars_config.spawns.get(&team.0).unwrap();
        pos.set(team_spawn.clone());

        commands
            .entity(entity)
            .insert(CombatState::default())
            .insert(FallingState::default());

        match_state
            .player_stats
            .insert(username.0.clone(), PlayerStats::default());

        let team = match_state.teams.get_mut(&team.0).unwrap();

        team.players.push(username.to_string());
    }

    commands.insert_resource(match_state);
}
