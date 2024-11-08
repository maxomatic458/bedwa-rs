use std::str::FromStr;

use valence::command::parsers::AbsoluteOrRelative;
use valence::layer::chunk::IntoBlock;
use valence::{command, command_macros, prelude::*};

use command::handler::CommandResultEvent;
use command::parsers;
use command_macros::Command;
use parsers::Vec3 as Vec3Parser;

use crate::bedwars_config::{ConfigVec3, SerBlock, WIPWorldConfig, WORLD_CONFIG_NAME};
use crate::colors::TeamColor;
use crate::utils::item_kind::ItemKindExtColor;

// TODO: in addition to the commands
// make this possible via a "edit mode"
// where the player can interact with the world

#[derive(Command, Debug, Clone)]
#[paths("bw-admin", "bedwars-admin", "bwa")]
#[scopes("bedwars.command.bw-admin")]
/// These commands will be used to setup a bedwars arena
pub enum BedwarsAdminCommand {
    /// Set the bounds of a bedwars arena (one arena is always bound to a world)
    #[paths = "arenabounds {pos1} {pos2}"]
    SetArenaBounds { pos1: Vec3Parser, pos2: Vec3Parser },
    /// TODO: team_color enum not working here
    /// Add a team to the bedwars arena
    #[paths = "team add {team_name} {team_color}"]
    AddTeam {
        team_name: String,
        team_color: String,
    },
    /// Remove a team from the bedwars arena
    #[paths = "team remove {team_name}"]
    RemoveTeam { team_name: String },
    /// Set the spawn point for a team
    #[paths = "team spawn {team_name} {pos}"]
    SetTeamSpawn { team_name: String, pos: Vec3Parser },
    /// Set the bed location for a team
    #[paths = "team bed {team_name} {pos}"]
    SetTeamBed { team_name: String, pos: Vec3Parser },
    /// Add a shop to the bedwars arena, and optionally bind it to a team
    /// (will only be accessible by that team)
    #[paths = "shop add {pos} {yaw} {team?}"]
    AddShop {
        pos: Vec3Parser,
        yaw: f32,
        team: Option<String>,
    },
    /// Add a resource spawner, and optionally bind it to a team
    /// If bound to a team, resources will stop spawning when the team is eliminated
    /// TODO: resource enum instead of string
    #[paths = "spawner add {pos} {resource} {interval} {amount} {team?}"]
    AddSpawner {
        pos: Vec3Parser,
        resource: String,
        interval: f32,
        amount: u32,
        team: Option<String>,
    },
    /// Remove a spawner from the bedwars arena
    #[paths = "spawner remove {pos}"]
    RemoveSpawner { pos: Vec3Parser },
    /// Remove a shop from the bedwars arena
    #[paths = "shop remove {pos}"]
    RemoveShop { pos: Vec3Parser },
    #[paths = "lobby spawn {pos}"]
    /// Set the lobby spawn point
    SetLobbySpawn { pos: Vec3Parser },
    #[paths = "spectator spawn {pos}"]
    /// Set the spectator spawn point
    SetSpectatorSpawn { pos: Vec3Parser },

    /// Print a summary of the bedwars arena
    #[paths = "summary"]
    Summary,
    #[paths = "help"]
    Help,

    /// Reset the bedwars arena config
    #[paths = "reset"]
    Reset,

    /// Save the bedwars arena config
    #[paths = "save"]
    Save,
}

pub fn handle_bedwars_admin_command(
    // Query the player entities to get their positions
    mut entities: Query<(Entity, &Position, &mut Client)>,
    mut events: EventReader<CommandResultEvent<BedwarsAdminCommand>>,
    mut wip_config: ResMut<WIPWorldConfig>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for event in events.read() {
        let caller = event.executor;

        let player_pos = *entities.get(caller).unwrap().1;
        let mut player_client = entities.get_mut(caller).unwrap().2;

        let layer = layer.single_mut();

        match &event.result {
            BedwarsAdminCommand::SetArenaBounds { pos1, pos2 } => {
                let pos1 = absolute_pos(pos1, &player_pos);
                let pos2 = absolute_pos(pos2, &player_pos);
                set_arena_bounds_command(&mut wip_config, player_client, pos1, pos2)
            }
            BedwarsAdminCommand::AddTeam {
                team_name,
                team_color,
            } => add_team_command(&mut wip_config, player_client, team_name, team_color),
            BedwarsAdminCommand::RemoveTeam { team_name } => {
                remove_team_command(&mut wip_config, player_client, team_name)
            }
            BedwarsAdminCommand::SetTeamSpawn { team_name, pos } => {
                let pos = absolute_pos(pos, &player_pos);
                set_team_spawn_command(&mut wip_config, player_client, team_name, pos)
            }
            BedwarsAdminCommand::SetTeamBed {
                team_name,
                pos: bed,
            } => {
                let pos = absolute_pos(bed, &player_pos);
                set_team_bed_command(&mut wip_config, player_client, team_name, layer, pos);
            }
            BedwarsAdminCommand::AddShop { pos, yaw, team } => {
                let pos = absolute_pos(pos, &player_pos);
                add_shop_command(&mut wip_config, player_client, pos, *yaw, team.as_ref())
            }
            BedwarsAdminCommand::AddSpawner {
                pos,
                resource,
                interval,
                amount,
                team,
            } => {
                let pos = absolute_pos(pos, &player_pos);
                add_spawner_command(
                    &mut wip_config,
                    player_client,
                    pos,
                    resource,
                    *interval,
                    *amount,
                    team,
                )
            }
            BedwarsAdminCommand::RemoveSpawner { pos } => {
                let pos = absolute_pos(pos, &player_pos);
                remove_spawner_command(&mut wip_config, player_client, pos)
            }
            BedwarsAdminCommand::RemoveShop { pos } => {
                let pos = absolute_pos(pos, &player_pos);
                remove_shop_command(&mut wip_config, player_client, pos)
            }
            BedwarsAdminCommand::SetLobbySpawn { pos } => {
                let pos = absolute_pos(pos, &player_pos);
                set_lobby_spawn_command(&mut wip_config, player_client, pos)
            }
            BedwarsAdminCommand::SetSpectatorSpawn { pos } => {
                let pos = absolute_pos(pos, &player_pos);
                set_spectator_spawn_command(&mut wip_config, player_client, pos)
            }
            BedwarsAdminCommand::Summary => bedwars_summary_command(&wip_config, player_client),
            BedwarsAdminCommand::Help => bedwars_help_command(player_client),
            BedwarsAdminCommand::Reset => {
                *wip_config = WIPWorldConfig::default();
                player_client.send_chat_message("§aBedwars arena config reset");
            }
            BedwarsAdminCommand::Save => bedwars_save_command(&wip_config, player_client),
        }
    }
}

/// [`BedwarsAdminCommand::SetArenaBounds`] command
fn set_arena_bounds_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos1: ConfigVec3,
    pos2: ConfigVec3,
) {
    player_client.send_chat_message(format!("§aSet arena bounds to §7{} §aand §7{}", pos1, pos2));
    wip_config.bounds = Some((pos1, pos2));
}

/// [`BedwarsAdminCommand::AddTeam`] command
fn add_team_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    team_name: &str,
    team_color: &str,
) {
    let team_color = match TeamColor::from_str(&team_color.to_lowercase()) {
        Ok(color) => color,
        Err(_) => {
            player_client.send_chat_message("§cInvalid team color");
            return;
        }
    };

    if wip_config.teams.contains_key(team_name) {
        player_client.send_chat_message("§cTeam already exists");
        return;
    }

    wip_config.teams.insert(team_name.to_string(), team_color);
    player_client.send_chat_message(format!(
        "§aAdded team §7{} §awith color {}{}",
        team_name,
        team_color.text_color(),
        team_color.to_string().to_uppercase()
    ));
}

/// [`BedwarsAdminCommand::RemoveTeam`] command
fn remove_team_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    team_name: &str,
) {
    if wip_config.teams.remove(team_name).is_none() {
        player_client.send_chat_message("§cTeam does not exist");
    }

    player_client.send_chat_message(format!("§aRemoved team §7{}", team_name));
}

/// [`BedwarsAdminCommand::SetTeamSpawn`] command
pub fn set_team_spawn_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    team_name: &str,
    spawn: ConfigVec3,
) {
    if !wip_config.teams.contains_key(team_name) {
        player_client.send_chat_message("§cTeam does not exist");
        return;
    }

    player_client.send_chat_message(format!(
        "§aSet spawn for team §7{} §ato §7{}",
        team_name, spawn
    ));
    wip_config.spawns.insert(team_name.to_string(), spawn);
}

/// [`BedwarsAdminCommand::SetTeamBed`] command
pub fn set_team_bed_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    team_name: &str,
    layer: Mut<'_, ChunkLayer>,
    bed: ConfigVec3,
) {
    if !wip_config.teams.contains_key(team_name) {
        player_client.send_chat_message("§cTeam does not exist");
        return;
    }

    let mut bed_blocks: Vec<(ConfigVec3, SerBlock)> = vec![];

    if let Some(block) = layer.block(BlockPos::new(bed.x, bed.y, bed.z)) {
        bed_blocks.push((bed.clone(), block.into_block().into()));
        if block.state.to_kind().to_item_kind().is_bed() {
            let neighbors = [
                BlockPos::new(bed.x + 1, bed.y, bed.z),
                BlockPos::new(bed.x - 1, bed.y, bed.z),
                BlockPos::new(bed.x, bed.y, bed.z + 1),
                BlockPos::new(bed.x, bed.y, bed.z - 1),
            ];

            for neighbor in neighbors {
                if let Some(neighbor_block) = layer.block(neighbor) {
                    if neighbor_block.state.to_kind().to_item_kind().is_bed() {
                        bed_blocks.push((
                            ConfigVec3 {
                                x: neighbor.x,
                                y: neighbor.y,
                                z: neighbor.z,
                            },
                            neighbor_block.into_block().into(),
                        ));
                    }
                }
            }
        } else {
            player_client
                .send_chat_message("§cBlock was not detected as bed, if thats the case ignore");
        }
    }

    player_client.send_chat_message(format!("§aSet bed for team §7{} §ato §7{}", team_name, bed));

    wip_config.beds.insert(team_name.to_string(), bed_blocks);
}

/// [`BedwarsAdminCommand::AddShop`] command
pub fn add_shop_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos: ConfigVec3,
    yaw: f32,
    team: Option<&String>,
) {
    if let Some(team) = team {
        if !wip_config.teams.contains_key(team) {
            player_client.send_chat_message("§cTeam does not exist");
            return;
        }
    }

    player_client.send_chat_message(format!("§aAdded shop at §7{}", pos));
    wip_config.shops.push(((pos, yaw), team.cloned()));
}

/// [`BedwarsAdminCommand::AddSpawner`] command
fn add_spawner_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos: ConfigVec3,
    resource: &str,
    interval: f32,
    amount: u32,
    team: &Option<String>,
) {
    if let Some(team) = team {
        if !wip_config.teams.contains_key(team) {
            player_client.send_chat_message("§cTeam does not exist");
            return;
        }
    }

    player_client.send_chat_message(format!("§aAdded resource spawner at §7{}", pos));
    // validate resource type
    if let Some(resource) = ItemKind::from_str(resource) {
        wip_config.resource_spawners.push((
            pos,
            ItemStack::new(resource, amount as i8, None).into(),
            interval,
            team.clone(),
        ));
        return;
    }

    player_client.send_chat_message("§cInvalid resource type");
}

/// [`BedwarsAdminCommand::RemoveSpawner`] command
fn remove_spawner_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos: ConfigVec3,
) {
    if !wip_config
        .resource_spawners
        .iter()
        .any(|(spawner_pos, _, _, _)| spawner_pos == &pos)
    {
        player_client.send_chat_message("§cResource spawner does not exist");
        return;
    }

    wip_config
        .resource_spawners
        .retain(|(spawner_pos, _, _, _)| spawner_pos != &pos);

    player_client.send_chat_message(format!("§aRemoved resource spawner at §7{}", pos));
}

/// [`BedwarsAdminCommand::RemoveShop`] command
fn remove_shop_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos: ConfigVec3,
) {
    if !wip_config
        .shops
        .iter()
        .any(|((shop_pos, _), _)| shop_pos == &pos)
    {
        player_client.send_chat_message("§cShop does not exist");
        return;
    }

    wip_config
        .shops
        .retain(|((shop_pos, _), _)| shop_pos != &pos);

    player_client.send_chat_message(format!("§aRemoved shop at §7{}", pos));
}

/// [`BedwarsAdminCommand::SetLobbySpawn`] command
pub fn set_lobby_spawn_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos: ConfigVec3,
) {
    player_client.send_chat_message(format!("§aSet lobby spawn to §7{}", pos));
    wip_config.lobby_spawn = Some(pos);
}

/// [`BedwarsAdminCommand::SetSpectatorSpawn`] command
pub fn set_spectator_spawn_command(
    wip_config: &mut WIPWorldConfig,
    mut player_client: Mut<'_, Client>,
    pos: ConfigVec3,
) {
    player_client.send_chat_message(format!("§aSet spectator spawn to §7{}", pos));
    wip_config.spectator_spawn = Some(pos);
}

/// [`BedwarsAdminCommand::Summary`] command
fn bedwars_summary_command(wip_config: &WIPWorldConfig, mut player_client: Mut<'_, Client>) {
    let mut message = "§aBedwars Arena Summary\n".to_string();

    message.push_str(&format!(
        "§aBounds: {}\n",
        wip_config
            .bounds
            .as_ref()
            .map_or("§cNot set".to_string(), |(pos1, pos2)| format!(
                "§7{} - {}",
                pos1, pos2
            ))
    ));

    message.push_str(&format!(
        "§aTeams: {}\n",
        wip_config
            .teams
            .iter()
            .map(|(team_name, color)| format!(
                "§7({}: {})",
                team_name,
                color.to_string().to_uppercase()
            ))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    message.push_str(&format!(
        "§aSpawns: {}\n",
        wip_config
            .spawns
            .iter()
            .map(|(team_name, pos)| format!("§7({}: {})", team_name, pos))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    message.push_str(&format!(
        "§aBeds (or bed blocks): {}\n",
        wip_config
            .beds
            .iter()
            .map(|(team_name, pos)| format!("§7({}: {:?})", team_name, pos))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    message.push_str(&format!(
        "§aShops: {}\n",
        wip_config
            .shops
            .iter()
            .map(|((pos, _), team)| format!(
                "§7{}: {}",
                team.as_ref()
                    .map_or("GLOBAL".to_string(), |team| team.clone()),
                pos
            ))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    message.push_str(&format!(
        "§aResource Spawners: {}\n",
        wip_config
            .resource_spawners
            .iter()
            .map(|(pos, resource, _interval, team)| format!(
                "§7{}: {} ({})",
                team.as_ref()
                    .map_or("GLOBAL".to_string(), |team| team.clone()),
                pos,
                Into::<ItemStack>::into(resource.clone()).item.to_str()
            ))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    message.push_str(&format!(
        "§aLobby Spawn: {}\n",
        wip_config
            .lobby_spawn
            .as_ref()
            .map_or("§cNot set".to_string(), |pos| format!("§7{}", pos))
    ));

    message.push_str(&format!(
        "§aSpectator Spawn: {}\n",
        wip_config
            .spectator_spawn
            .as_ref()
            .map_or("§cNot set".to_string(), |pos| format!("§7{}", pos))
    ));

    player_client.send_chat_message(message);

    if wip_config.is_finished() {
        player_client.send_chat_message("§2Bedwars arena is ready to be saved!");
    } else {
        player_client.send_chat_message("§cBedwars arena is not ready to be saved!");
    }
}

/// [`BedwarsAdminCommand::Help`] command
fn bedwars_help_command(_player_client: Mut<'_, Client>) {
    todo!()
}

/// [`BedwarsAdminCommand::Save`] command
fn bedwars_save_command(wip_config: &WIPWorldConfig, mut player_client: Mut<'_, Client>) {
    if !wip_config.is_finished() {
        player_client.send_chat_message("§cBedwars arena is not ready to be saved!");
        return;
    }

    std::fs::write(
        WORLD_CONFIG_NAME,
        serde_json::to_string_pretty(wip_config).unwrap(),
    )
    .unwrap();
    player_client.send_chat_message("§aBedwars arena saved!");
}

fn absolute_pos(command_pos: &Vec3Parser, player_pos: &DVec3) -> ConfigVec3 {
    let x = match command_pos.x {
        AbsoluteOrRelative::Absolute(x) => x as f64,
        AbsoluteOrRelative::Relative(x) => player_pos.x + x as f64,
    };

    let y = match command_pos.y {
        AbsoluteOrRelative::Absolute(y) => y as f64,
        AbsoluteOrRelative::Relative(y) => player_pos.y + y as f64,
    };

    let z = match command_pos.z {
        AbsoluteOrRelative::Absolute(z) => z as f64,
        AbsoluteOrRelative::Relative(z) => player_pos.z + z as f64,
    };

    ConfigVec3 {
        x: x as i32,
        y: y as i32,
        z: z as i32,
    }
}
