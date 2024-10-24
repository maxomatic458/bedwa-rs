use std::collections::HashMap;

use rand::Rng;
use valence::{
    interact_block::InteractBlockEvent,
    layer::chunk::IntoBlock,
    prelude::*,
    protocol::{packets::play::BlockEventS2c, sound::SoundCategory, Sound, WritePacket},
    Layer,
};

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_chest_open)
            .insert_resource(ChestState::default())
            .observe(chest_close);
    }
}

#[derive(Resource, Default)]
pub struct ChestState {
    /// Position of chest -> (Inventory, number of players looking into the chest)
    pub chests: HashMap<BlockPos, (Inventory, u32)>,
    pub enderchests: HashMap<String, (Inventory, u32)>,
}

#[derive(Debug, Component)]
struct PlayerChestState {
    pub open_chest: BlockPos,
    pub is_ender_chest: bool,
}

// right-click chest
fn on_chest_open(
    mut commands: Commands,
    mut events: EventReader<InteractBlockEvent>,
    mut players: Query<(Entity, &Username)>,
    mut layer: Query<&mut ChunkLayer>,
    mut chest_state: ResMut<ChestState>,
) {
    for event in events.read() {
        let mut layer = layer.single_mut();
        let Some(block) = layer.block(event.position) else {
            continue;
        };

        let block = block.into_block();

        let Ok((player, player_name)) = players.get_mut(event.client) else {
            continue;
        };

        let default_inv = Inventory::new(InventoryKind::Generic9x3);

        let ((inventory, mut players_looking_into_chest), sound) = match block.state.to_kind() {
            BlockKind::Chest => (
                chest_state
                    .chests
                    .get(&event.position)
                    .unwrap_or(&(default_inv, 0))
                    .clone(),
                Sound::BlockChestOpen,
            ),
            BlockKind::EnderChest => (
                chest_state
                    .enderchests
                    .get(&player_name.0)
                    .unwrap_or(&(default_inv, 0))
                    .clone(),
                Sound::BlockEnderChestOpen,
            ),
            _ => continue,
        };

        players_looking_into_chest += 1;

        let inv_id = commands.spawn(inventory.clone()).id();
        commands.entity(player).insert(OpenInventory::new(inv_id));

        commands.entity(player).insert(PlayerChestState {
            open_chest: event.position,
            is_ender_chest: block.state.to_kind() == BlockKind::EnderChest,
        });

        if block.state.to_kind() == BlockKind::EnderChest {
            chest_state.enderchests.insert(
                player_name.0.clone(),
                (inventory.clone(), players_looking_into_chest),
            );
        } else {
            chest_state.chests.insert(
                event.position,
                (inventory.clone(), players_looking_into_chest),
            );
        }

        layer
            .view_writer(event.position)
            .write_packet(&BlockEventS2c {
                position: event.position,
                action_id: 1,
                action_parameter: players_looking_into_chest as u8,
                block_type: block.state.to_kind(),
            });

        layer.play_sound(
            sound,
            SoundCategory::Block,
            DVec3::new(
                event.position.x as f64,
                event.position.y as f64,
                event.position.z as f64,
            ),
            0.5,
            rand::thread_rng().gen_range(0.9..=1.),
        );
    }
}

fn chest_close(
    trigger: Trigger<OnRemove, OpenInventory>,
    mut commands: Commands,
    players: Query<(Entity, &PlayerChestState, &OpenInventory, &Username)>,
    inventories: Query<&Inventory, Without<PlayerChestState>>,
    mut chest_state: ResMut<ChestState>,
    mut layer: Query<&mut ChunkLayer>,
) {
    let Ok((player_ent, player_chest_state, open_inventory, player_name)) =
        players.get(trigger.entity())
    else {
        return;
    };

    let Ok(chest_inventory) = inventories.get(open_inventory.entity) else {
        return;
    };

    let mut layer = layer.single_mut();

    let (res, block_kind, close_sound) = if player_chest_state.is_ender_chest {
        (
            chest_state.enderchests.get_mut(&player_name.0),
            BlockKind::EnderChest,
            Sound::BlockEnderChestClose,
        )
    } else {
        (
            chest_state.chests.get_mut(&player_chest_state.open_chest),
            BlockKind::Chest,
            Sound::BlockChestClose,
        )
    };

    let Some((inv, players_looking_into_chest)) = res else {
        return;
    };

    *inv = chest_inventory.clone();
    *players_looking_into_chest -= 1;

    layer
        .view_writer(player_chest_state.open_chest)
        .write_packet(&BlockEventS2c {
            position: player_chest_state.open_chest,
            action_id: 1,
            action_parameter: *players_looking_into_chest as u8,
            block_type: block_kind,
        });

    layer.play_sound(
        close_sound,
        SoundCategory::Block,
        DVec3::new(
            player_chest_state.open_chest.x as f64,
            player_chest_state.open_chest.y as f64,
            player_chest_state.open_chest.z as f64,
        ),
        0.5,
        rand::thread_rng().gen_range(0.9..=1.),
    );

    commands.entity(player_ent).remove::<PlayerChestState>();
}
