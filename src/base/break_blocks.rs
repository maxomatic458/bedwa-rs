use action::{DiggingEvent, DiggingState};
use app::{App, Plugin, Update};
use bevy_state::prelude::in_state;
use client::Username;
use entity::{
    entity::NoGravity,
    item::{ItemEntityBundle, Stack},
    EntityLayerId, Position, Velocity,
};
use math::{DVec3, Vec3};
use prelude::{Commands, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Query, Res};
use rand::Rng;
use valence::*;

use crate::{bedwars_config::WorldConfig, utils::despawn_timer::DespawnTimer, GameState, Team};

use super::{
    build::PlayerPlacedBlocks,
    item_pickup::PickupMarker,
    physics::{CollidesWithBlocks, GetsStuckOnCollision, Gravity, PhysicsMarker},
};
/// Strength of random velocity applied to the dropped item after breaking a block
const BLOCK_BREAK_DROP_STRENGTH: f32 = 0.05 * 20.0;

pub struct BlockBreakPlugin;

#[derive(Debug, Event, PartialEq)]
pub struct BedDestroyedEvent {
    pub attacker: Entity,
    pub team: Team,
}

impl Plugin for BlockBreakPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (break_blocks,).run_if(in_state(GameState::Match)))
            .add_event::<BedDestroyedEvent>();
    }
}

#[allow(clippy::too_many_arguments)]
fn break_blocks(
    mut commands: Commands,
    clients: Query<(&Username, &Team)>,
    mut events: EventReader<DiggingEvent>,
    mut layer: Query<(Entity, &mut ChunkLayer)>,
    player_placed_blocks: Res<PlayerPlacedBlocks>,
    bedwars_config: Res<WorldConfig>,
    // match_state: ResMut<MatchState>,
    mut event_writer: EventWriter<BedDestroyedEvent>,
) {
    for event in events.read() {
        let (layer, mut layer_mut) = layer.single_mut();
        if event.state != DiggingState::Stop {
            continue;
        }

        let block_pos = event.position;

        let Ok((_player_name, player_team)) = clients.get(event.client) else {
            continue;
        };

        let mut broke_bed = false;

        for (team_name, bed_block_set) in &bedwars_config.beds {
            if *team_name == player_team.name {
                continue;
            }

            let block_pos_vec = crate::bedwars_config::ConfigVec3 {
                x: block_pos.x,
                y: block_pos.y,
                z: block_pos.z,
            };

            if bed_block_set.iter().any(|(pos, _)| pos == &block_pos_vec) {
                // set bed to broken
                for (pos, _block) in bed_block_set {
                    layer_mut.set_block(BlockPos::new(pos.x, pos.y, pos.z), BlockState::AIR);
                }

                let (victim_team, victim_color) =
                    bedwars_config.teams.get_key_value(team_name).unwrap();

                event_writer.send(BedDestroyedEvent {
                    attacker: event.client,
                    team: Team {
                        name: victim_team.clone(),
                        color: *victim_color,
                    },
                });

                broke_bed = true;
            }
        }

        if let Some(block_state) = player_placed_blocks.0.get(&block_pos) {
            if broke_bed {
                continue;
            }

            let item_stack = ItemStack {
                item: block_state.to_kind().to_item_kind(),
                count: 1,
                nbt: None,
            };

            let mut rng = rand::thread_rng();

            let position = DVec3 {
                x: block_pos.x as f64 + 0.5 + rng.gen_range(-0.1..0.1),
                y: block_pos.y as f64 + 0.5 + rng.gen_range(-0.1..0.1),
                z: block_pos.z as f64 + 0.5 + rng.gen_range(-0.1..0.1),
            };

            let item_velocity = Vec3 {
                x: rng.gen_range(-1.0..1.0),
                y: rng.gen_range(-1.0..1.0),
                z: rng.gen_range(-1.0..1.0),
            } * BLOCK_BREAK_DROP_STRENGTH;

            commands
                .spawn(ItemEntityBundle {
                    item_stack: Stack(item_stack),
                    position: Position(position),
                    velocity: Velocity(item_velocity),

                    layer: EntityLayerId(layer),
                    entity_no_gravity: NoGravity(true),
                    ..Default::default()
                })
                .insert(PickupMarker::default())
                .insert(Gravity::items())
                .insert(PhysicsMarker)
                .insert(CollidesWithBlocks(None))
                .insert(GetsStuckOnCollision::ground())
                .insert(DespawnTimer::items());

            layer_mut.set_block(block_pos, BlockState::AIR);
        }
    }
}
