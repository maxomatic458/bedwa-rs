use std::thread::yield_now;

use bevy_state::prelude::in_state;
use bevy_time::{Time, Timer, TimerMode};
use valence::{command::parsers::gamemode, entity::living::Health, prelude::*};
use bevy_ecs::query::{Added, With};
use valence::prelude::Inventory;

use crate::{bedwars_config::{self, BedwarsConfig}, r#match::MatchState, utils::inventory, GameState, Team};
#[derive(Debug, Clone, Component)]
pub struct IsDead;

pub struct OnDeathPlugin;
#[derive(Debug, Clone, Component)]
pub struct RespawnTimerPlayer(pub Timer);
impl Plugin for OnDeathPlugin { 
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(Update, (ondeath.run_if(in_state(GameState::Match)),tick_player_respawn_timer))
        .observe(player_respawn);
    } 
    
}

fn tick_player_respawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut clients: Query<
    (
        Entity,
        &mut RespawnTimerPlayer
    )
>,
){
    for (player, mut timer) in &mut clients {
        timer.0.tick(time.delta());
        if timer.0.finished(){
            commands.entity(player).remove::<IsDead>();
            commands.entity(player).remove::<RespawnTimerPlayer>();
        }
        

    }
}

fn player_respawn(
    _trigger: Trigger<OnRemove, IsDead>,
   mut clients: Query<(
        &mut Position,
        &mut Health,
        &mut GameMode,
        &Team
    )
>, bedwars_config:Res<BedwarsConfig>
){
    for (mut player_pos, mut player_hea, mut player_gm, team) in &mut clients {
        player_hea.0 = 20.0;
        *player_gm = GameMode::Survival;
        let team_spawn_pos = bedwars_config.spawns.get(&team.0).unwrap();
        player_pos.set(DVec3::new(team_spawn_pos.x as f64, team_spawn_pos.y as f64, team_spawn_pos.z as f64))
    }
}



fn ondeath(
    mut commands: Commands,
    mut clients: Query<
   (
    Entity,
    &mut Inventory,
    &mut GameMode,
   &Team
   ),
   Added<IsDead>
   
   >,
   match_state:Res<MatchState>
){
    for (player, mut inventory, mut game_mode, team) in &mut clients {
        let bed_destroyed = match_state.teams.get(&team.0).unwrap().bed_destroyed;
        *game_mode = GameMode::Spectator;
        if bed_destroyed == true {

            
        }
        else {
            let player_respawn_timer = RespawnTimerPlayer(Timer::from_seconds(5.0, TimerMode::Once));
            commands.entity(player).insert(player_respawn_timer);
        

            }  
            
        
    
}
}