use bevy_ecs::bundle::Bundle;
use bevy_state::{prelude::in_state, state::OnEnter};
use bevy_time::{Time, Timer, TimerMode};
use valence::{
    entity::{
        entity::NoGravity,
        item::{ItemEntityBundle, Stack},
    },
    prelude::*,
    ItemStack,
};

use crate::{
    base::item_pickup::PickupMarker, bedwars_config::BedwarsConfig, r#match::MatchState, GameState,
    Team,
};

pub struct ResourceSpawnerPlugin;

impl Plugin for ResourceSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Match), (init_resource_spawners,))
            .add_systems(Update, spawn_resources.run_if(in_state(GameState::Match)));
    }
}

#[derive(Debug, Bundle)]
struct ResourceSpawnerBundle {
    marker: ResourceSpawner,
    position: Position,
}

#[derive(Debug, Component)]
struct ResourceSpawner {
    item: ItemStack,
    timer: Timer,
}

fn init_resource_spawners(mut commands: Commands, bedwars_config: Res<BedwarsConfig>) {
    for (pos, ser_item_stack, interval_sec, team_name) in &bedwars_config.resource_spawners {
        let spawner_ent = commands
            .spawn(ResourceSpawnerBundle {
                marker: ResourceSpawner {
                    item: Into::<ItemStack>::into(ser_item_stack.clone()),
                    timer: Timer::from_seconds(*interval_sec, TimerMode::Repeating),
                },
                position: Position([pos.x as f64, pos.y as f64, pos.z as f64].into()),
            })
            .id();

        if let Some(team_name) = team_name {
            let team_color = bedwars_config.teams.get(team_name).unwrap();
            commands.entity(spawner_ent).insert(Team {
                name: team_name.clone(),
                color: *team_color,
            });
        }
    }
}

fn spawn_resources(
    mut commands: Commands,
    mut spawners: Query<(Entity, &mut ResourceSpawner, &Position, Option<&Team>)>,
    match_state: Res<MatchState>,
    time: Res<Time>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (spawner_ent, mut spawner, pos, team) in &mut spawners {
        if spawner.timer.tick(time.delta()).just_finished() {
            if let Some(team) = team {
                if match_state.teams.get(&team.name).unwrap().bed_destroyed {
                    commands.entity(spawner_ent).insert(Despawned);
                    continue;
                }
            }

            let layer = layers.single();

            // adjust pos to center of block
            let pos = DVec3::new(pos.0.x - 0.5, pos.0.y, pos.0.z - 0.5);

            commands
                .spawn(ItemEntityBundle {
                    item_stack: Stack(spawner.item.clone()),
                    layer: EntityLayerId(layer),
                    position: Position(pos),
                    entity_no_gravity: NoGravity(true),
                    ..Default::default()
                })
                .insert(PickupMarker);
        }
    }
}
