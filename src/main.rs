use std::num::NonZero;

use base::{
    armor_right_click_equip::ArmorRightClickEquipPlugin,
    bow::BowPlugin,
    break_blocks::BlockBreakPlugin,
    build::BuildPlugin,
    chat::ChatPlugin,
    chests::ChestPlugin,
    combat::CombatPlugin,
    death::{DeathPlugin, PlayerEliminatedEvent},
    drop_items::ItemDropPlugin,
    fall_damage::FallDamagePlugin,
    item_pickup::ItemPickupPlugin,
    physics::PhysicsPlugin,
    regeneration::RegenerationPlugin,
    scoreboard::ScoreboardPlugin,
    utils::debug::DebugPlugin,
    void_death::VoidDeathPlugin,
};
use bevy_state::{app::StatesPlugin, prelude::*};
use bevy_time::{Time, TimePlugin};
use colors::TeamColor;
use commands::bedwars_admin::{handle_bedwars_admin_command, BedwarsAdminCommand};
use edit::EditPlugin;
use items::ender_pearl::EnderPearlPlugin;
use lobby::{LobbyPlayerState, LobbyPlugin};
use menu::ItemMenuPlugin;
use r#match::MatchPlugin;
use resource_spawners::ResourceSpawnerPlugin;
// use resource_spawners::ResourceSpawnerPlugin;
use shop::ShopPlugin;
use spectator::SpectatorPlugin;
use utils::despawn_timer::DespawnTimerPlugin;
use valence::{anvil::AnvilLevel, command::AddCommand, prelude::*, ServerSettings};

pub mod base;
pub mod bedwars_config;
pub mod colors;
pub mod commands;
pub mod edit;
pub mod items;
pub mod lobby;
pub mod r#match;
pub mod menu;
pub mod resource_spawners;
pub mod shop;
pub mod spectator;
pub mod utils;

/// A component that will be attached to players in the lobby
#[derive(Debug, Default, Component)]
pub struct LobbyPlayer;
/// A component that will be attached to players spectating a match
#[derive(Debug, Default, Component)]
pub struct Spectator;

/// A component that will be attached to players that are still playing
// #[derive(Debug, Default, Component)]
// pub struct ActivePlayer;

/// A component that will be attached to players that are editing the map
#[derive(Debug, Default, Component)]
pub struct Editor;

/// A component that will be attached to players in a match
#[derive(Debug, Clone, Component, PartialEq, Eq, Hash)]
pub struct Team {
    pub name: String,
    pub color: TeamColor,
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum GameState {
    #[default]
    Lobby,
    Match,
    PostMatch,
    Edit,
}

// Time stamp when the last tick finished
#[derive(Debug, Default, Resource)]
pub struct LastTickTime(pub std::time::Duration);

fn main() {
    std::env::set_var("RUST_LOG", "debug");

    App::new()
        .insert_resource(ServerSettings {
            tick_rate: NonZero::new(20).unwrap(),
            ..Default::default()
        })
        .add_plugins(StatesPlugin)
        .add_plugins(ChatPlugin)
        .add_plugins(SpectatorPlugin)
        .add_plugins(ScoreboardPlugin)
        .add_plugins(EditPlugin)
        .add_plugins(VoidDeathPlugin)
        .add_plugins(DeathPlugin)
        .add_plugins(TimePlugin)
        .add_plugins(FallDamagePlugin)
        .init_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(LobbyPlugin)
        .add_plugins(BuildPlugin)
        .add_plugins(BlockBreakPlugin)
        .add_plugins(ItemMenuPlugin)
        .add_plugins(MatchPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(ItemPickupPlugin)
        .add_plugins(RegenerationPlugin)
        .add_plugins(BowPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(EnderPearlPlugin)
        // .add_plugins(ItemEntityPlugin)
        .add_plugins(DespawnTimerPlugin)
        .add_plugins(ItemDropPlugin)
        .add_plugins(ResourceSpawnerPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(ArmorRightClickEquipPlugin)
        .add_plugins(ChestPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                handle_bedwars_admin_command,
                update_last_tick_time,
                despawn_disconnected_clients,
            ),
        )
        // DEBUG
        .add_plugins(DebugPlugin)
        .add_command::<BedwarsAdminCommand>()
        .insert_resource(LastTickTime::default())
        .observe(on_disconnect)
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
    mut state: ResMut<NextState<GameState>>,
) {
    let layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    let level = AnvilLevel::new("world", &biomes);

    commands.spawn((layer, level));

    let wip_config = {
        if let Ok(config) = bedwars_config::load_config() {
            commands.insert_resource(config.clone());
            bedwars_config::WIPWorldConfig::from_saved_config(&config)
        } else {
            tracing::warn!("No bedwars config found, enabling edit mode");
            state.set(GameState::Edit);
            bedwars_config::WIPWorldConfig::default()
        }
    };

    commands.insert_resource(wip_config);

    let shop_config = {
        bedwars_config::load_trader_config().unwrap_or_else(|e| {
            tracing::error!("Failed to load trader config: {}", e);
            std::process::exit(1);
        })
    };

    commands.insert_resource(shop_config);
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
    // bedwars_config: Option<Res<bedwars_config::BedwarsConfig>>,
    state: Res<State<GameState>>,
) {
    let layer = layers.single();
    for (entity, mut layer_id, mut visible_chunk_layer, mut visible_entity_layers) in &mut clients {
        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);

        match state.get() {
            GameState::Lobby => {
                commands.entity(entity).insert(LobbyPlayer);
            }
            GameState::Match | GameState::PostMatch => {
                commands.entity(entity).insert(Spectator);
            }
            GameState::Edit => {
                commands.entity(entity).insert(Editor);
            } // _ => {}
        };
    }
}

fn on_disconnect(
    trigger: Trigger<OnRemove, Client>,
    query: Query<(&Username, &Position)>,
    // commands: Commands,
    game_state: Res<State<GameState>>,
    mut lobby_state: Option<ResMut<LobbyPlayerState>>,
    // eliminate a player that disconnects during a match
    mut elimination_writer: EventWriter<PlayerEliminatedEvent>,
) {
    let Ok((username, position)) = query.get(trigger.entity()) else {
        return;
    };

    tracing::info!("Player {} disconnected", username);

    match game_state.get() {
        GameState::Lobby => {
            if let Some(lobby_state) = lobby_state.as_mut() {
                lobby_state.players.remove(&username.0);
                lobby_state.without_team = lobby_state.without_team.saturating_sub(1);
            }
        }
        GameState::Match => {
            elimination_writer.send(PlayerEliminatedEvent {
                attacker: None,
                victim: trigger.entity(),
                position: position.0,
            });
        }
        _ => {}
    }
}

fn update_last_tick_time(time: Res<Time>, mut last_tick_time: ResMut<LastTickTime>) {
    last_tick_time.0 = time.delta();
}
