use bevy_state::prelude::in_state;
use valence::{
    abilities::PlayerAbilitiesFlags,
    command::scopes::CommandScopes,
    entity::living::Health,
    interact_block::InteractBlockEvent,
    interact_item::InteractItemEvent,
    inventory::{player_inventory::PlayerInventory, HeldItem},
    message::SendMessage,
    nbt::compound,
    prelude::*,
    protocol::{sound::SoundCategory, Sound},
};

use crate::{
    bedwars_config::BedwarsWIPConfig,
    commands::bedwars_admin::{
        add_shop_command, set_lobby_spawn_command, set_spectator_spawn_command,
        set_team_bed_command, set_team_spawn_command,
    },
    menu::{ItemMenu, MenuItemSelectEvent},
    Editor, GameState, Team,
};

const SET_TEAM_BED_ITEM_NAME: &str = "SET TEAM BED BLOCK(S)";
const SET_TEAM_SPAWN_ITEM_NAME: &str = "SET TEAM SPAWN";
const SET_SHOP_ITEM_NAME: &str = "SET SHOP";
const TEAM_CHANGER_ITEM_NAME: &str = "CHANGE SELECTED TEAM";
const NO_SPECIFIC_TEAM_TEXT: &str = "No specific team";
const TEAM_ONLY_NOTICE: &str = "(only when team is alive)";
const LOBBY_SPAWN_ITEM_NAME: &str = "SET LOBBY SPAWN";
const SPECTATOR_SPAWN_ITEM_NAME: &str = "SET SPECTATOR SPAWN";

pub struct EditPlugin;

impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_editor,
                edit_right_click,
                edit_click_block,
                on_change_team_mode,
            )
                .run_if(in_state(GameState::Edit)),
        );
    }
}

#[allow(clippy::type_complexity)]
fn init_editor(
    mut clients: Query<
        (
            &mut Client,
            &mut Position,
            &mut Inventory,
            &mut Health,
            &mut CommandScopes,
            &mut PlayerAbilitiesFlags,
        ),
        Added<Editor>,
    >,
) {
    for (mut client, mut position, mut inventory, mut health, mut scopes, mut abilities) in
        &mut clients
    {
        position.set([0.5, 128.0, 0.5]);
        inventory.readonly = true;
        health.0 = 20.0;

        abilities.set_allow_flying(true);
        scopes.add("bedwars.command.bw-admin");

        // Team selector
        inventory.set_slot(
            PlayerInventory::hotbar_to_slot(8),
            ItemStack::new(
                ItemKind::Compass,
                1,
                Some(compound! {
                    "display" => compound! {
                        "Name" => format!(
                            "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                            "§l§7", // Bold
                            TEAM_CHANGER_ITEM_NAME
                        )
                    }
                }),
            ),
        );

        // Villager shop spawn
        inventory.set_slot(
            PlayerInventory::hotbar_to_slot(2),
            ItemStack::new(
                ItemKind::VillagerSpawnEgg,
                1,
                Some(compound! {
                    "display" => compound! {
                        "Name" => format!(
                            "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                            "§l§7", // Bold
                            SET_SHOP_ITEM_NAME
                        )
                    }
                }),
            ),
        );

        // Lobby spawn
        inventory.set_slot(
            PlayerInventory::hotbar_to_slot(6),
            ItemStack::new(
                ItemKind::Book,
                1,
                Some(compound! {
                    "display" => compound! {
                        "Name" => format!(
                            "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                            "§l§7", // Bold
                            LOBBY_SPAWN_ITEM_NAME
                        )
                    }
                }),
            ),
        );

        // Spectator spawn
        inventory.set_slot(
            PlayerInventory::hotbar_to_slot(7),
            ItemStack::new(
                ItemKind::EnderPearl,
                1,
                Some(compound! {
                    "display" => compound! {
                        "Name" => format!(
                            "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                            "§l§7", // Bold
                            SPECTATOR_SPAWN_ITEM_NAME,
                        )
                    }
                }),
            ),
        );

        client.send_chat_message("§l§6You are currently in edit mode!");
    }
}

fn edit_right_click(
    mut commands: Commands,
    mut clients: Query<(&mut Client, &HeldItem), With<Editor>>,
    mut events: EventReader<InteractItemEvent>,
    wip_config: Res<BedwarsWIPConfig>,
) {
    for event in events.read() {
        let Ok((mut client, held_item)) = clients.get_mut(event.client) else {
            continue;
        };

        if held_item.hotbar_idx() == 8 {
            open_team_selection_menu(&mut commands, event.client, &mut client, &wip_config)
        }
    }
}

fn open_team_selection_menu(
    commands: &mut Commands,
    player_ent: Entity,
    client: &mut Client,
    wip_config: &BedwarsWIPConfig,
) {
    if wip_config.teams.is_empty() {
        client.send_chat_message("§4There are no teams available!");
        client.send_chat_message("§aTo add a team run /bwa team add <team name> <team color>");
        return;
    }

    let mut inv = Inventory::new(InventoryKind::Generic9x3);
    // TODO: set item name to team name

    inv.set_slot(
        0,
        ItemStack::new(
            ItemKind::Barrier,
            1,
            Some(compound! {
                "display" => compound! {
                    "Name" => format!(
                        "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                        "§l§7", // Bold
                        NO_SPECIFIC_TEAM_TEXT
                    )
                }
            }),
        ),
    );

    for (team_name, color) in &wip_config.teams {
        let next_slot = inv.first_empty_slot().unwrap();

        let team_block = color.wool_block();
        let item_stack = ItemStack::new(
            team_block,
            1,
            Some(compound! {
                "display" => compound! {
                    "Name" => format!(
                        "{{\"text\":\"{}{}{}\", \"italic\": \"false\"}}'}}",
                        "§l", // Bold
                        color.text_color(),
                        team_name
                    )
                }
            }),
        );

        inv.set_slot(next_slot, item_stack);
    }

    let menu = ItemMenu::new(inv);
    commands.entity(player_ent).insert(menu);
}

fn on_change_team_mode(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Client, &mut Inventory, &Position), With<Editor>>,
    mut events: EventReader<MenuItemSelectEvent>,
    wip_config: Res<BedwarsWIPConfig>,
) {
    for event in events.read() {
        let Ok((player_ent, mut client, mut inventory, position)) = clients.get_mut(event.client)
        else {
            continue;
        };
        let selected_slot = event.idx;

        if selected_slot == 0 {
            // No specific team mode
            commands.entity(player_ent).remove::<ItemMenu>();

            inventory.set_slot(PlayerInventory::hotbar_to_slot(0), ItemStack::EMPTY);
            inventory.set_slot(PlayerInventory::hotbar_to_slot(1), ItemStack::EMPTY);
            // TODO: code duplication
            inventory.set_slot(
                PlayerInventory::hotbar_to_slot(2),
                ItemStack::new(
                    ItemKind::VillagerSpawnEgg,
                    1,
                    Some(compound! {
                        "display" => compound! {
                            "Name" => format!(
                                "{{\"text\":\"{}{}\", \"italic\": \"false\"}}'}}",
                                "§l§7", // Bold
                                SET_SHOP_ITEM_NAME
                            )
                        }
                    }),
                ),
            );

            continue;
        }

        if let Some((team, team_color)) = wip_config.teams.iter().nth(selected_slot as usize - 1) {
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

            commands.entity(player_ent).remove::<ItemMenu>();

            inventory.set_slot(
                PlayerInventory::hotbar_to_slot(0),
                ItemStack::new(
                    team_color.bed_block(),
                    1,
                    Some(compound! {
                        "display" => compound! {
                            "Name" => format!(
                                "{{\"text\":\"{}{}{}\", \"italic\": \"false\"}}'}}",
                                "§l", // Bold
                                team_color.text_color(),
                                SET_TEAM_BED_ITEM_NAME
                            )
                        }
                    }),
                ),
            );

            inventory.set_slot(
                PlayerInventory::hotbar_to_slot(1),
                ItemStack::new(
                    team_color.wool_carpet(),
                    1,
                    Some(compound! {
                        "display" => compound! {
                            "Name" => format!(
                                "{{\"text\":\"{}{}{}\", \"italic\": \"false\"}}'}}",
                                "§l", // Bold
                                team_color.text_color(),
                                SET_TEAM_SPAWN_ITEM_NAME
                            )
                        }
                    }),
                ),
            );

            inventory.set_slot(
                PlayerInventory::hotbar_to_slot(2),
                ItemStack::new(
                    ItemKind::VillagerSpawnEgg,
                    1,
                    Some(compound! {
                        "display" => compound! {
                            "Name" => format!(
                                "{{\"text\":\"{}{}{} {}\", \"italic\": \"false\"}}'}}",
                                "§l",
                                team_color.text_color(),
                                SET_SHOP_ITEM_NAME,
                                TEAM_ONLY_NOTICE,
                            )
                        }
                    }),
                ),
            );
        }
    }
}

fn edit_click_block(
    mut clients: Query<(&mut Client, &mut Inventory, &Look, &HeldItem, Option<&Team>)>,
    mut events: EventReader<InteractBlockEvent>,
    mut layers: Query<&mut ChunkLayer>,
    mut wip_config: ResMut<BedwarsWIPConfig>,
) {
    for event in events.read() {
        let Ok((client, mut inventory, look, held_item, team)) = clients.get_mut(event.client)
        else {
            continue;
        };

        if event.hand != Hand::Main {
            continue;
        }

        let layer = layers.single_mut();

        // TODO: do a PR on valence main for this
        // builder inventory is readonly
        // change the bitfield to held_item.slot

        let slot_id = held_item.slot();
        inventory.changed |= 1 << slot_id;

        let config_vec_pos = crate::bedwars_config::ConfigVec3 {
            x: event.position.x,
            y: event.position.y,
            z: event.position.z,
        };

        match held_item.hotbar_idx() {
            // Select bed block(s)
            0 => {
                if let Some(team) = team {
                    set_team_bed_command(
                        &mut wip_config,
                        client,
                        &team.name,
                        layer,
                        config_vec_pos,
                    );
                }
            }
            // place spawn block
            1 => {
                if let Some(team) = team {
                    set_team_spawn_command(
                        &mut wip_config,
                        client,
                        &team.name,
                        config_vec_pos + crate::bedwars_config::ConfigVec3::up(),
                    );
                }
            }
            // place shop
            2 => {
                // we want to round the player's yaw to the nearest 45 degrees
                let yaw = look.yaw;
                let yaw = (yaw / 45.0).round() * 45.0 + 180.0;

                add_shop_command(
                    &mut wip_config,
                    client,
                    config_vec_pos + crate::bedwars_config::ConfigVec3::up(),
                    yaw,
                    team.map(|t| t.name.clone()).as_ref(),
                );
            }
            // TODO: team chest
            3 => {}
            // TODO: ender chest
            4 => {}

            // lobby spawn
            6 => {
                set_lobby_spawn_command(
                    &mut wip_config,
                    client,
                    config_vec_pos + crate::bedwars_config::ConfigVec3::up(),
                );
            }
            // spectator spawn
            7 => {
                set_spectator_spawn_command(
                    &mut wip_config,
                    client,
                    config_vec_pos + crate::bedwars_config::ConfigVec3::up(),
                );
            }
            _ => {}
        }
    }
}
