use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::{Added, With},
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_state::{prelude::in_state, state::NextState};
use bevy_time::{Timer, TimerMode};
use valence::{
    app::{App, Plugin, Update},
    client::{Client, Username},
    entity::{living::Health, Position},
    interact_item::InteractItemEvent,
    inventory::HeldItem,
    nbt::{compound, Compound},
    player_list::DisplayName,
    prelude::{Component, IntoSystemConfigs, Inventory, InventoryKind},
    protocol::{sound::SoundCategory, Sound},
    title::SetTitle,
    GameMode, ItemKind, ItemStack,
};

use crate::{
    bedwars_config::{self, WorldConfig},
    menu::{ItemMenu, MenuItemSelectEvent},
    utils::inventory::InventoryExt,
    GameState, LobbyPlayer, Team,
};

const TEAM_SELECTOR_ITEM: ItemKind = ItemKind::Compass;
const _TEAM_SELECTOR_ITEM_NAME: &str = "Team Selector";
const NO_TEAM_SELECTED_ITEM: ItemKind = ItemKind::Barrier;
/// TODO: create a lobby plugin

pub struct LobbyPlugin;

/// Represents the players in the lobby
#[derive(Debug, Clone, Resource, Default)]
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
                init_lobby_player.run_if(in_state(GameState::Lobby)),
                lobby_right_click.run_if(in_state(GameState::Lobby)),
                on_team_select.run_if(in_state(GameState::Lobby)),
                set_action_bar,
            ),
        )
        .insert_resource(LobbyPlayerState::default());
    }
}

fn init_lobby_player(
    mut clients: Query<
        (&mut Position, &mut GameMode, &mut Inventory, &mut Health),
        (Added<LobbyPlayer>,),
    >,
    mut lobby_state: ResMut<LobbyPlayerState>,
    bedwars_config: Res<bedwars_config::WorldConfig>,
) {
    for (mut position, mut game_mode, mut inventory, mut health) in &mut clients {
        tracing::info!("Initializing lobby player");
        position.set(bedwars_config.lobby_spawn.clone());
        *game_mode = GameMode::Survival;
        *health = Health(20.0);

        inventory.clear();
        inventory.readonly = true;

        let team_selector_nbt = Compound::new();

        // TODO: custom item name

        let team_selector = ItemStack::new(TEAM_SELECTOR_ITEM, 1, Some(team_selector_nbt));

        inventory.set_slot(36, team_selector);

        let no_team_selected_item = ItemStack::new(NO_TEAM_SELECTED_ITEM, 1, None);
        inventory.set_slot(40, no_team_selected_item);

        // commands.entity(player).insert(CombatState::default());

        lobby_state.without_team += 1;
    }
}

fn lobby_right_click(
    mut commands: Commands,
    clients: Query<(&HeldItem, &Inventory), With<LobbyPlayer>>,
    mut events: EventReader<InteractItemEvent>,
    bedwars_config: Res<WorldConfig>,
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
    player_ent: Entity,
    bedwars_config: &WorldConfig,
) {
    let mut inv = Inventory::new(InventoryKind::Generic9x1);
    // TODO: set item name to team name
    for (team_name, color) in &bedwars_config.teams {
        let next_slot = inv.first_empty_slot().unwrap();

        let team_block = color.wool_block();
        let item_stack = ItemStack::new(
            team_block,
            1,
            Some(compound! {
                "display" => compound! {
                    "Name" => format!(
                        "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                        color.text_color(),
                        team_name,
                    )
                }
            }),
        );
        inv.set_slot(next_slot, item_stack);
    }

    let menu = ItemMenu::new(inv);
    commands.entity(player_ent).insert(menu);
}

#[allow(clippy::type_complexity)]
fn on_team_select(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &Position,
            &mut Client,
            &Username,
            Option<&Team>,
            &mut DisplayName,
        ),
        With<LobbyPlayer>,
    >,
    mut events: EventReader<MenuItemSelectEvent>,
    mut lobby_state: ResMut<LobbyPlayerState>,
    mut state: ResMut<NextState<GameState>>,
    bedwars_config: Res<WorldConfig>,
) {
    for event in events.read() {
        let Ok((player_ent, position, mut client, username, switched_from, mut display_name)) =
            clients.get_mut(event.client)
        else {
            continue;
        };
        let selected_slot = event.idx;

        if let Some((team, team_color)) = bedwars_config.teams.iter().nth(selected_slot as usize) {
            commands.entity(player_ent).insert(Team {
                name: team.to_string(),
                color: *team_color,
            });

            client.play_sound(
                Sound::BlockNoteBlockBell,
                SoundCategory::Master,
                position.0,
                1.0,
                1.0,
            );

            display_name.0 = Some(format!("{}{}", team_color.text_color(), username).into());

            commands.entity(player_ent).remove::<ItemMenu>();

            client.set_action_bar(format!("{}{} team", team_color.text_color(), team));

            commands
                .entity(player_ent)
                .insert(TeamActionBarTimer::default());

            lobby_state.players.insert(username.0.clone(), team.clone());
            if switched_from.is_none() {
                lobby_state.without_team = lobby_state.without_team.saturating_sub(1);
            }

            if lobby_state.without_team == 0 {
                tracing::info!("setting state to match");
                for (player, _, _, _, _, _) in clients.iter() {
                    commands.entity(player).remove::<LobbyPlayer>();
                }

                state.set(GameState::Match);
            }
        }
    }
}

const ACTION_BAR_UPDATE_INTERVAL_SEC: f32 = 2.0;
#[derive(Debug, Component)]
struct TeamActionBarTimer(pub Timer);

impl Default for TeamActionBarTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            ACTION_BAR_UPDATE_INTERVAL_SEC,
            TimerMode::Repeating,
        ))
    }
}

/// This will be called every 2 seconds,
/// to update the action bar for each player
fn set_action_bar(mut players: Query<(&mut Client, &Team, &TeamActionBarTimer)>) {
    for (mut client, team, timer) in players.iter_mut() {
        if !timer.0.just_finished() {
            continue;
        }


        let team_color = team.color;
        client.set_action_bar(&format!("{} Team: {}", team_color.text_color(), team.name));
    }
}
