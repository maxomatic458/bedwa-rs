use valence::prelude::*;

use crate::{bedwars_config::BedwarsConfig, Spectator};

pub struct SpectatorPlugin;

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (init_spectator,));
    }
}

fn init_spectator(
    // commands: Commands,
    mut clients: Query<(Entity, &mut Position), Added<Spectator>>,
    bedwars_config: Res<BedwarsConfig>,
) {
    for (_player_ent, mut position) in clients.iter_mut() {
        position.set(Into::<DVec3>::into(bedwars_config.spectator_spawn.clone()));
    }
}
