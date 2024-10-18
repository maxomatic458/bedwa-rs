use base::{
    break_blocks::BlockBreakPlugin, build::BuildPlugin, chat::ChatPlugin, combat::CombatPlugin,
    death::DeathPlugin, drop_items::ItemDropPlugin, fall_damage::FallDamagePlugin,
    item_pickup::ItemPickupPlugin, void_death::VoidDeathPlugin,
};
use bevy_state::{app::StatesPlugin, prelude::*};
use bevy_time::{Time, TimePlugin};
use colors::TeamColor;
use commands::bedwars_admin::{handle_bedwars_admin_command, BedwarsAdminCommand};
use edit::EditPlugin;
use lobby::LobbyPlugin;
use menu::ItemMenuPlugin;
use r#match::MatchPlugin;
use resource_spawners::ResourceSpawnerPlugin;
// use resource_spawners::ResourceSpawnerPlugin;
use shop::ShopPlugin;
use spectator::SpectatorPlugin;
use utils::despawn_timer::DespawnTimerPlugin;
use valence::{anvil::AnvilLevel, command::AddCommand, prelude::*};

pub mod base;
pub mod bedwars_config;
pub mod colors;
pub mod commands;
pub mod edit;
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
#[derive(Debug, Default, Component)]
pub struct ActivePlayer;

/// A component that will be attached to players that are editing the map
#[derive(Debug, Default, Component)]
pub struct Editor;

/// A component that will be attached to players in a match
#[derive(Debug, Clone, Component, PartialEq)]
pub struct Team {
    pub name: String,
    pub color: TeamColor,
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum GameState {
    #[default]
    Lobby,
    Match,
    // PostMatch,
    Edit,
}

/// The time the last tick took
#[derive(Debug, Default, Resource)]
pub struct LastTickTime(pub std::time::Duration);

fn main() {
    std::env::set_var("RUST_LOG", "debug");

    App::new()
        .add_plugins(StatesPlugin)
        .add_plugins(ChatPlugin)
        .add_plugins(SpectatorPlugin)
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
        // .add_plugins(ItemEntityPlugin)
        .add_plugins(DespawnTimerPlugin)
        .add_plugins(ItemDropPlugin)
        .add_plugins(ResourceSpawnerPlugin)
        .add_plugins(CombatPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                handle_bedwars_admin_command,
                update_last_tick_time,
            ),
        )
        .add_command::<BedwarsAdminCommand>()
        .insert_resource(LastTickTime::default())
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

    // let mut wip_config = bedwars_config::BedwarsWIPConfig::default();

    // if let Ok(config) = bedwars_config::load_config() {
    //     wip_config = bedwars_config::BedwarsWIPConfig::from_saved_config(&config);
    //     tracing::info!("Using saved bedwars config");
    //     commands.insert_resource(config);
    // } else {
    //     tracing::warn!("No bedwars config found, using default");
    // }

    let wip_config = {
        if let Ok(config) = bedwars_config::load_config() {
            commands.insert_resource(config.clone());
            bedwars_config::BedwarsWIPConfig::from_saved_config(&config)
        } else {
            tracing::warn!("No bedwars config found, enabling edit mode");
            state.set(GameState::Edit);
            bedwars_config::BedwarsWIPConfig::default()
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

        // if let Some(ref config) = bedwars_config {
        match state.get() {
            GameState::Lobby => {
                commands.entity(entity).insert(LobbyPlayer);
            }
            GameState::Match => {
                commands.entity(entity).insert(Spectator);
            }
            GameState::Edit => {
                commands.entity(entity).insert(Editor);
            } // _ => {}
        };
        // } else {
        //     pos.set([0.5, 128.0, 0.5]);
        //     *game_mode = GameMode::Creative;
        //     scopes.add("bedwars.command.bw-admin");
        //     client.send_chat_message(
        //         "§cThe bedwars config is not loaded, you are currently in the edit mode.",
        //     );
        // }
    }
}

fn update_last_tick_time(time: Res<Time>, mut last_tick_time: ResMut<LastTickTime>) {
    // tracing::debug!("Last tick time: {:?}", time.delta());
    last_tick_time.0 = time.delta();
}
