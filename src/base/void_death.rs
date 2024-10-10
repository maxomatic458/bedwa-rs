use bevy_ecs::{entity::Entity, query::{With, Without}, system::{Commands, Query, Res}};
use color_eyre::owo_colors::colors::Yellow;
use valence::{app::{Plugin, Update}, entity::{display::Width, Position}, prelude::Inventory, protocol::packets::play::player_abilities_s2c, GameMode};

use crate::{bedwars_config, utils::inventory, Team};

use super::on_death::IsDead;

pub struct VoidDeathPlugin;

impl Plugin for VoidDeathPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(Update, voiddeath);
    }
}

fn voiddeath(
    mut commands: Commands,
    mut clients: Query<
   (
    Entity,
    &mut Position,
    &mut GameMode,
    &mut Inventory,
   ),
   (With<Team>, Without<IsDead>)
   >, 
   bedwars_config: Res<bedwars_config::BedwarsConfig>,
){ 
    for (player, mut position, mut game_mode, mut inventory) in &mut clients {
        let void = &bedwars_config.bounds.0.y.min(bedwars_config.bounds.1.y);
        if position.y < *void as f64 {
           commands.entity(player).insert(IsDead);
        } 

        //*game_mode = GameMode::Spectator;
    }

}
