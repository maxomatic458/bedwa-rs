use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res},
};
use bevy_state::prelude::in_state;
use valence::{
    app::{Plugin, Update},
    entity::Position,
    prelude::IntoSystemConfigs,
};

use crate::{bedwars_config, GameState, Team};

use super::death::IsDead;

// TODO: here arena bounds?

pub struct VoidDeathPlugin;

impl Plugin for VoidDeathPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(Update, void_death.run_if(in_state(GameState::Match)));
    }
}

#[allow(clippy::type_complexity)]
fn void_death(
    mut commands: Commands,
    mut clients: Query<(Entity, &Position), (With<Team>, Without<IsDead>)>,
    bedwars_config: Res<bedwars_config::BedwarsConfig>,
) {
    for (player, position) in &mut clients {
        let void = &bedwars_config.bounds.0.y.min(bedwars_config.bounds.1.y);
        if position.y < *void as f64 {
            commands.entity(player).insert(IsDead);
        }
    }
}
