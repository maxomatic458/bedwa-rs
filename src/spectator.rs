use bevy_state::prelude::in_state;
use valence::prelude::*;

use crate::{bedwars_config::WorldConfig, GameState, Spectator};

pub struct SpectatorPlugin;

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (init_spectator,).run_if(not(in_state(GameState::Edit))),
        );
    }
}

fn init_spectator(
    // commands: Commands,
    mut clients: Query<(Entity, &mut Position, &mut GameMode), Added<Spectator>>,
    bedwars_config: Res<WorldConfig>,
) {
    for (_player_ent, mut position, mut game_mode) in clients.iter_mut() {
        *game_mode = GameMode::Spectator;
        position.set(Into::<DVec3>::into(bedwars_config.spectator_spawn.clone()));
    }
}
