use action::{DiggingEvent, DiggingState};
use app::{App, Plugin, Update};
use entity::{
    entity::NoGravity,
    item::{ItemEntityBundle, Stack},
    EntityLayerId, Position, Velocity,
};
use math::{DVec3, Vec3};
use prelude::{Commands, Entity, EventReader, Query, Res};
use rand::Rng;
use valence::*;

use crate::utils::{block_state::BlockStateExt, despawn_timer::DespawnTimer};

use super::{build::PlayerPlacedBlocks, drop_items::DroppedItemsPickupTimer};

/// Strength of random velocity applied to the dropped item after breaking a block
const BLOCK_BREAK_DROP_STRENGTH: f32 = 0.05 * 20.0;

pub struct BlockBreakPlugin;

impl Plugin for BlockBreakPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (break_blocks,));
    }
}

fn break_blocks(
    mut commands: Commands,
    mut events: EventReader<DiggingEvent>,
    mut layer: Query<(Entity, &mut ChunkLayer)>,
    player_placed_blocks: Res<PlayerPlacedBlocks>,
) {
    let (layer, mut layer_mut) = layer.single_mut();

    for event in events.read() {
        if event.state != DiggingState::Stop {
            continue;
        }

        let block_pos = event.position;
        if let Some(block_state) = player_placed_blocks.0.get(&block_pos) {
            if block_state.is_bed() {
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
