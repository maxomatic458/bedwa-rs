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
use prelude::{
    Commands, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Query, Res, ResMut,
};
use rand::Rng;
use valence::*;

use crate::{
    bedwars_config::BedwarsConfig, r#match::MatchState, utils::despawn_timer::DespawnTimer,
    GameState, Team,
};

use super::{build::PlayerPlacedBlocks, drop_items::DroppedItemsPickupTimer};
use crate::utils::item_kind::ItemKindExtColor;
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
    bedwars_config: Res<BedwarsConfig>,
    mut match_state: ResMut<MatchState>,
    mut event_writer: EventWriter<BedDestroyedEvent>,
) {
    let (layer, mut layer_mut) = layer.single_mut();

    for event in events.read() {
        if event.state != DiggingState::Stop {
            continue;
        }

        let block_pos = event.position;

        let Ok((_player_name, player_team)) = clients.get(event.client) else {
            continue;
        };

        for (team_name, bed_block_set) in &bedwars_config.beds {
            if *team_name == player_team.name {
                continue;
            }

            let block_pos_vec = crate::bedwars_config::ConfigVec3 {
                x: block_pos.x,
                y: block_pos.y,
                z: block_pos.z,
            };

            if bed_block_set.contains(&block_pos_vec) {
                // set bed to broken
                for block in bed_block_set {
                    layer_mut.set_block(BlockPos::new(block.x, block.y, block.z), BlockState::AIR);
                }

                let victim_team_color = bedwars_config.teams.get(team_name).unwrap();

                let victim_team_state = match_state.teams.get_mut(team_name).unwrap();
                victim_team_state.bed_destroyed = true;

                event_writer.send(BedDestroyedEvent {
                    attacker: event.client,
                    team: Team {
                        name: team_name.clone(),
                        color: *victim_team_color,
                    },
                });
            }
        }

        if let Some(block_state) = player_placed_blocks.0.get(&block_pos) {
            if block_state.to_kind().to_item_kind().is_bed() {
                // extra logic for breaking beds
            } else {
                // we want to drop the item

                let item_stack = ItemStack {
                    item: block_state.to_kind().to_item_kind(),
                    count: 1,
                    nbt: None,
                };

                // the item should have some random
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
                        // entity_air
                        ..Default::default()
                    })
                    .insert(DroppedItemsPickupTimer::default())
                    .insert(DespawnTimer::from_secs(2.0));
            }

            layer_mut.set_block(block_pos, BlockState::AIR);
        }
    }
}
