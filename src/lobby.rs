use std::{collections::HashMap, time::Instant};

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::{Added, With},
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_state::state::NextState;
use valence::{
    app::{App, Plugin, Update},
    client::{Client, Username},
    entity::{living::Health, Position},
    interact_item::InteractItemEvent,
    inventory::HeldItem,
    nbt::Compound,
    prelude::{Inventory, InventoryKind, OpenInventory},
    protocol::{sound::SoundCategory, Sound},
    title::SetTitle,
    GameMode, ItemKind, ItemStack,
};

use crate::{
    bedwars_config::{self, BedwarsConfig},
    menu::{ItemMenu, MenuItemSelect},
    GameState, LobbyPlayer, Team,
};

const TEAM_SELECTOR_ITEM: ItemKind = ItemKind::Compass;
const TEAM_SELECTOR_ITEM_NAME: &str = "Team Selector";
const NO_TEAM_SELECTED_ITEM: ItemKind = ItemKind::Barrier;
/// TODO: create a lobby plugin

pub struct LobbyPlugin;

/// Represents the players in the lobby
#[derive(Debug, Clone, Resource)]
pub struct LobbyPlayerState {
    /// player name -> team name
    pub players: HashMap<String, String>,
    /// Number of players that did not select a team
    /// If this is 0, the game can start
    pub without_team: u16,
}
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_lobby_player,
                lobby_right_click,
                on_team_select,
                set_action_bar,
            ),
        )
        .insert_resource(LastUpdatedActionBar(Instant::now()))
        .insert_resource(LobbyPlayerState {
            players: HashMap::new(),
            without_team: 0,
        });
    }
}

fn init_lobby_player(
    commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut Position,
            &mut GameMode,
            &mut Inventory,
            &mut Health,
        ),
        Added<LobbyPlayer>,
    >,
    mut lobby_state: ResMut<LobbyPlayerState>,
    bedwars_config: Res<bedwars_config::BedwarsConfig>,
) {
    for (player, mut position, mut game_mode, mut inventory, mut health) in &mut clients {
        tracing::info!("Initializing lobby player");
        position.set(bedwars_config.lobby_spawn.clone());
        *game_mode = GameMode::Survival;
        *health = Health(20.0);

        inventory.readonly = true;

        let team_selector_nbt = Compound::new();

        // TODO: custom item name

        let team_selector = ItemStack::new(TEAM_SELECTOR_ITEM, 1, Some(team_selector_nbt));

        inventory.set_slot(36, team_selector);

        let no_team_selected_item = ItemStack::new(NO_TEAM_SELECTED_ITEM, 1, None);
        inventory.set_slot(40, no_team_selected_item);

        let stack_of_stone = ItemStack::new(ItemKind::Dirt, 64, None);
        inventory.set_slot(41, stack_of_stone);

        // commands.entity(player).insert(CombatState::default());

        lobby_state.without_team += 1;
    }
}

fn lobby_right_click(
    mut commands: Commands,
    clients: Query<(&HeldItem, &Inventory), With<LobbyPlayer>>,
    mut events: EventReader<InteractItemEvent>,
    bedwars_config: Res<BedwarsConfig>,
) {
    for event in events.read() {
        for (held_item, inventory) in &mut clients.iter() {
            let held_item_slot = held_item.slot();
            if inventory.slot(held_item_slot).item == TEAM_SELECTOR_ITEM {
                tracing::info!("Team selector right click");
                open_team_selection_menu(&mut commands, event.client, &bedwars_config);
            }
        }
    }
}

fn open_team_selection_menu(
    commands: &mut Commands,
    player: Entity,
    bedwars_config: &BedwarsConfig,
) {
    let mut inv = Inventory::new(InventoryKind::Generic9x3);
    // TODO: set item name to team name
    for (_team_name, color) in &bedwars_config.teams {
        let next_slot = inv.first_empty_slot().unwrap();

        let team_block = color.wool_block();
        let item_stack = ItemStack::new(team_block, 1, None);
        inv.set_slot(next_slot, item_stack);
    }

    // while let Some(next_free_slot) = inv.first_empty_slot() {
    //     let item_stack = ItemStack::new(ItemKind::Stone, 1, None);
    //     inv.set_slot(next_free_slot, item_stack);
    // }

    let menu = ItemMenu::new(inv);
    commands.entity(player).insert(menu);
}

fn on_team_select(
    mut commands: Commands,
    mut players: Query<(Entity, &Position, &mut Client, &Username), With<LobbyPlayer>>,
    mut events: EventReader<MenuItemSelect>,
    mut lobby_state: ResMut<LobbyPlayerState>,
    mut state: ResMut<NextState<GameState>>,
    bedwars_config: Res<BedwarsConfig>,
) {
    for event in events.read() {
        let selected_slot = event.idx;
        for (player, pos, mut client, username) in players.iter_mut() {
            if player != event.client {
                continue;
            }

            if let Some((team, team_color)) =
                bedwars_config.teams.iter().nth(selected_slot as usize)
            {
                commands.entity(player).insert(Team(team.to_string()));
                client.play_sound(
                    Sound::BlockNoteBlockBell,
                    SoundCategory::Master,
                    pos.0,
                    1.0,
                    1.0,
                );

                commands.entity(player).remove::<OpenInventory>();

                client.set_action_bar(format!("{}{} team", team_color.text_color(), team));

                tracing::info!("Player {} selected team {}", username, team);

                lobby_state.players.insert(username.0.clone(), team.clone());
                lobby_state.without_team -= 1;

                if lobby_state.without_team == 0 {
                    tracing::info!("setting state to match");
                    commands.entity(player).remove::<LobbyPlayer>();
                    state.set(GameState::Match);
                }
            }
        }
    }
}

#[derive(Debug, Resource)]
struct LastUpdatedActionBar(pub Instant);

/// This will be called every 2 seconds,
/// to update the action bar for each player
fn set_action_bar(
    mut players: Query<(&mut Client, &Team), With<Team>>,
    bedwars_config: Res<BedwarsConfig>,
    mut last_run: ResMut<LastUpdatedActionBar>,
) {
    let now = Instant::now();
    if now.duration_since(last_run.0).as_secs() < 2 {
        return;
    }

    last_run.0 = now;

    for (mut client, team) in players.iter_mut() {
        let team_color = bedwars_config.teams.get(&team.0).unwrap();
        client.set_action_bar(&format!("{} Team: {}", team_color.text_color(), team.0));
    }
}
