// use bevy_ecs::{
//     bundle::Bundle,
//     entity::Entity,
//     query::With,
//     system::{Commands, Query, Res},
// };
// use bevy_state::state::OnEnter;
// use valence::{
//     app::App,
//     entity::{
//         item::{ItemEntityBundle, Stack},
//         EntityLayerId, Position,
//     },
//     prelude::{Component, Plugin},
//     ChunkLayer, EntityLayer, ItemKind, ItemStack,
// };

// use crate::{base::drop_items::DroppedItemsPickupTimer, bedwars_config::BedwarsConfig, GameState};

// pub struct ResourceSpawnerPlugin;

// #[derive(Debug, Bundle)]
// struct ResourceSpawner {
//     last_spawn: LastSpawn,
//     resource: Resource,
//     interval: SpawnInterval,
//     amount_per_spawn: AmountPerSpawn,
// }

// #[derive(Debug, Clone, Component)]
// struct LastSpawn(pub std::time::Instant);

// #[derive(Debug, Clone, Component)]
// struct Resource(pub ItemKind);

// #[derive(Debug, Clone, Component)]
// struct SpawnInterval(pub std::time::Duration);

// // #[derive(Debug, Clone, Component)]
// // struct AmountPerSpawn(pub u16);

// impl Plugin for ResourceSpawnerPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(OnEnter(GameState::Match), (init_resource_spawners,));
//         // )).add_systems(Update, (
//         //     // spawn_resources,
//         // ));
//     }
// }

// fn init_resource_spawners(
//     mut commands: Commands,
//     bedwars_config: Res<BedwarsConfig>,
//     // layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
// ) {
//     // let layer = layers.single();
//     for (pos, _team) in &bedwars_config.resource_spawners {
//         let mut entity_commands = commands.spawn(ResourceSpawner {
//             last_spawn: LastSpawn(std::time::Instant::now()),
//             resource: Resource(ItemKind::IronIngot),
//             interval: SpawnInterval(std::time::Duration::from_secs(3)),
//             amount_per_spawn: AmountPerSpawn(1),
//         });

//         entity_commands.insert(valence::entity::Position(pos.clone().into()));
//     }
// }

// fn spawn_resources(
//     mut commands: Commands,
//     mut spawners: Query<(
//         &Resource,
//         &mut LastSpawn,
//         &SpawnInterval,
//         &AmountPerSpawn,
//         &Position,
//     )>,
//     layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
// ) {
//     let layer = layers.single();
//     for (resource, mut last_spawn, interval, amount, pos) in spawners.iter_mut() {
//         if last_spawn.0.elapsed() >= interval.0 {
//             last_spawn.0 = std::time::Instant::now();

//             tracing::info!("spawning resource at: {:?}", pos);

//             commands
//                 .spawn(ItemEntityBundle {
//                     // kind: EntityKind::ITEM,
//                     item_stack: Stack(ItemStack {
//                         item: resource.0,
//                         count: amount.0 as i8,
//                         nbt: None,
//                     }),
//                     layer: EntityLayerId(layer),
//                     position: *pos,
//                     ..Default::default()
//                 })
//                 .insert(DroppedItemsPickupTimer::default());
//             tracing::info!("Spawned resource");
//         }
//     }
// }
