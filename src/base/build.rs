use std::collections::HashMap;

use valence::{
    entity::living::LivingEntity, interact_block::InteractBlockEvent, inventory::HeldItem,
    math::Aabb, prelude::*,
};

use crate::bedwars_config;

pub struct BuildPlugin;

#[derive(Debug, Default, Resource)]
pub struct PlayerPlacedBlocks(pub HashMap<BlockPos, BlockState>);

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (place_blocks,))
            .insert_resource(PlayerPlacedBlocks::default());
    }
}

fn place_blocks(
    mut clients: Query<(&mut Inventory, &HeldItem)>,
    entities: Query<&Hitbox, With<LivingEntity>>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
    bedwars_config: Res<bedwars_config::BedwarsConfig>,
    mut player_placed_blocks: ResMut<PlayerPlacedBlocks>,
) {
    let mut layer = layers.single_mut();

    for event in events.read() {
        let Ok((mut inventory, held)) = clients.get_mut(event.client) else {
            continue;
        };

        if event.hand != Hand::Main {
            continue;
        }

        // get the held item
        let slot_id = held.slot();
        let stack = inventory.slot(slot_id);
        if stack.is_empty() {
            continue;
        };

        let Some(block_kind) = BlockKind::from_item_kind(stack.item) else {
            continue;
        };

        let block_state = BlockState::from_kind(block_kind);
        // TODO: check for bedwars arena bounds
        let block_hitboxes = block_state.collision_shapes();
        let real_pos = event.position.get_in_direction(event.face);

        if let Some(block) = layer.block(real_pos) {
            if !block.state.is_air() {
                return;
            }
        }

        for mut block_hitbox in block_hitboxes {
            let tolerance = DVec3::new(0.0, 0.005, 0.0);

            block_hitbox = Aabb::new(
                block_hitbox.min()
                    + DVec3::new(real_pos.x as f64, real_pos.y as f64, real_pos.z as f64)
                    + tolerance,
                block_hitbox.max()
                    + DVec3::new(real_pos.x as f64, real_pos.y as f64, real_pos.z as f64)
                    - tolerance,
            );

            // TODO: also very inefficient
            for entity_hitbox in entities.iter() {
                tracing::info!("Checking entity hitbox");
                tracing::info!(
                    "block: {:?} intersects entity: {:?}",
                    block_hitbox,
                    entity_hitbox
                );
                if block_hitbox.intersects(**entity_hitbox) {
                    return;
                }
            }
        }

        if stack.count > 1 {
            let amount = stack.count - 1;
            inventory.set_slot_amount(slot_id, amount);
        } else {
            inventory.set_slot(slot_id, ItemStack::EMPTY);
        }

        let state = block_kind.to_state().set(
            PropName::Axis,
            match event.face {
                Direction::Down | Direction::Up => PropValue::Y,
                Direction::North | Direction::South => PropValue::Z,
                Direction::West | Direction::East => PropValue::X,
            },
        );

        player_placed_blocks.0.insert(real_pos, state);

        layer.set_block(real_pos, state);
        tracing::error!("placed at: {}", real_pos);
    }
}
