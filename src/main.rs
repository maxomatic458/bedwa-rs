use base::{
    break_blocks::BlockBreakPlugin, build::BuildPlugin, combat::CombatPlugin,
    drop_items::ItemDropPlugin, item::ItemEntityPlugin, item_pickup::ItemPickupPlugin,
};
use bevy_state::{app::StatesPlugin, prelude::*};
use bevy_time::{Time, TimePlugin};
use commands::bedwars_admin::{handle_bedwars_admin_command, BedwarsAdminCommand};
use lobby::LobbyPlugin;
use menu::{ItemMenuPlugin, MenuItemSelect};
use r#match::MatchPlugin;
use resource_spawners::ResourceSpawnerPlugin;
use shop::ShopPlugin;
use utils::despawn_timer::DespawnTimerPlugin;
use valence::{
    anvil::AnvilLevel,
    command::{scopes::CommandScopes, AddCommand},
    prelude::*,
};

pub mod base;
pub mod bedwars_config;
pub mod colors;
pub mod commands;
pub mod lobby;
pub mod r#match;
pub mod menu;
pub mod resource_spawners;
pub mod shop;
pub mod utils;

/// A component that will be attached to players in the lobby
#[derive(Debug, Default, Component)]
pub struct LobbyPlayer;
/// A component that will be attached to players spectating a match
#[derive(Debug, Default, Component)]
pub struct SpectatorPlayer;
/// A component that will be attached to players in a match
#[derive(Debug, Component)]
pub struct Team(pub String);

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum GameState {
    #[default]
    Lobby,
    Match,
    PostMatch,
}

/// The time the last tick took
#[derive(Debug, Default, Resource)]
pub struct LastTickTime(pub std::time::Duration);

fn main() {
    std::env::set_var("RUST_LOG", "debug");

    App::new()
        .add_plugins(StatesPlugin)
        .add_plugins(TimePlugin)
        .init_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(LobbyPlugin)
        .add_plugins(BuildPlugin)
        .add_plugins(BlockBreakPlugin)
        .add_plugins(ItemMenuPlugin)
        .add_plugins(MatchPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(ItemPickupPlugin)
        .add_plugins(ItemEntityPlugin)
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
        .add_event::<MenuItemSelect>()
        .insert_resource(LastTickTime::default())
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
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
            tracing::warn!("No bedwars config found, using default");
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

fn init_clients(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut CommandScopes,
            &mut Position,
            &mut GameMode,
            &mut Client,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
    bedwars_config: Option<Res<bedwars_config::BedwarsConfig>>,
    state: Res<State<GameState>>,
) {
    let layer = layers.single();
    for (
        entity,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut scopes,
        mut pos,
        mut game_mode,
        mut client,
    ) in &mut clients
    {
        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);

        if let Some(ref config) = bedwars_config {
            match state.get() {
                GameState::Lobby => {
                    commands.entity(entity).insert(LobbyPlayer);
                }
                GameState::Match => {
                    commands.entity(entity).insert(SpectatorPlayer);
                }
                _ => {}
            };
        } else {
            pos.set([0.5, 128.0, 0.5]);
            *game_mode = GameMode::Creative;
            scopes.add("bedwars.command.bw-admin");
            client.send_chat_message(
                "§cThe bedwars config is not loaded, you are currently in the edit mode.",
            );
        }
    }
}

fn update_last_tick_time(time: Res<Time>, mut last_tick_time: ResMut<LastTickTime>) {
    // tracing::debug!("Last tick time: {:?}", time.delta());
    last_tick_time.0 = time.delta();
}
