use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::{With, Without},
    system::{Commands, Query, Res},
};
use bevy_state::{prelude::in_state, state::OnEnter};
use valence::{
    app::{Plugin, Update},
    client::{Client, Username},
    entity::{player::PlayerEntity, EntityLayerId, HeadYaw, Look, Position},
    prelude::{
        Component, DetectChangesMut, InteractEntityEvent, IntoSystemConfigs, Inventory,
        InventoryKind,
    },
    protocol::{sound::SoundCategory, Sound},
    ChunkLayer, EntityLayer, ItemKind, ItemStack,
};

use crate::{
    base::death::IsDead,
    bedwars_config::{ShopConfig, WorldConfig},
    menu::{ItemMenu, MenuItemSelectEvent},
    utils::inventory::InventoryExt,
    GameState, Team,
};

const SHOP_INVENTORY_TYPE: InventoryKind = InventoryKind::Generic9x5;

#[derive(Debug, Clone, Component)]
pub struct Shop;

#[derive(Component, Default)]
pub struct ShopState {
    selected_category: Option<String>,
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(OnEnter(GameState::Match), (init_shops,))
            .add_systems(
                Update,
                (on_shop_click, on_shop_open).run_if(in_state(GameState::Match)),
            );
    }
}

/// Initialize the shops
fn init_shops(
    mut commands: Commands,
    bedwars_config: Res<WorldConfig>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    tracing::debug!("initializing shops");
    let layer = layers.single();
    for ((pos, yaw), team) in &bedwars_config.shops {
        // TODO: why use villager as entity ?
        let mut entity_commands = commands.spawn(valence::entity::villager::VillagerEntityBundle {
            layer: EntityLayerId(layer),
            position: valence::entity::Position(
                [pos.x as f64 + 0.5, pos.y as f64, pos.z as f64 + 0.5].into(),
            ),
            head_yaw: HeadYaw(*yaw),
            look: Look::new(*yaw, 0.0),
            ..Default::default()
        });

        entity_commands.insert(Shop);

        tracing::debug!("Initialized shop at {:?}", pos);
        if let Some(team) = team {
            let team_color = bedwars_config.teams.get(team.as_str()).unwrap();
            entity_commands.insert(Team {
                name: team.clone(),
                color: *team_color,
            });
        }
    }
}

/// The player opens the shop by left or right clicking the villager
#[allow(clippy::type_complexity)]
fn on_shop_open(
    mut commands: Commands,
    mut events: EventReader<InteractEntityEvent>,
    mut players: Query<(Entity, &mut Client, &Username), (With<PlayerEntity>, Without<IsDead>)>,
    shops: Query<&Position, With<Shop>>,
    shop_config: Res<ShopConfig>,
) {
    for event in events.read() {
        let Ok((player_ent, mut client, username)) = players.get_mut(event.client) else {
            continue;
        };

        let Ok(shop_position) = shops.get(event.entity) else {
            continue;
        };

        let shop_menu = main_menu_from_shop_config(&shop_config);

        tracing::debug!("{username} opened shop");

        client.play_sound(
            Sound::EntityVillagerAmbient,
            SoundCategory::Neutral,
            shop_position.0,
            0.5,
            1.0,
        );

        commands
            .entity(player_ent)
            .insert(shop_menu)
            .insert(ShopState::default());
    }
}

fn main_menu_from_shop_config(shop_config: &ShopConfig) -> ItemMenu {
    let mut shop_menu = Inventory::new(InventoryKind::Generic9x5);
    for (idx, (_category_name, (category_item, _))) in shop_config.shop_items.iter().enumerate() {
        // TODO: add category_name via nbt
        shop_menu.set_slot(idx as u16, category_item.clone());
    }

    ItemMenu::new(shop_menu)
}

fn on_shop_click(
    mut inventories: Query<&mut Inventory, Without<Client>>,
    mut clients: Query<(
        &mut Client,
        &Position,
        &mut Inventory,
        &mut ShopState,
        &Team,
        &ItemMenu,
    )>,
    mut events: EventReader<MenuItemSelectEvent>,
    shop_config: Res<ShopConfig>,
    bedwars_config: Res<WorldConfig>,
) {
    for event in events.read() {
        let Ok((mut client, position, mut inventory, mut shop_state, team, item_menu)) =
            clients.get_mut(event.client)
        else {
            continue;
        };

        let Some(inventory_ent) = item_menu.inventory_ent() else {
            continue;
        };

        let Ok(mut menu_inventory) = inventories.get_mut(inventory_ent) else {
            continue;
        };

        let team_color = bedwars_config.teams.get(&team.name).unwrap();

        let select_index = event.idx;

        let category = shop_state.selected_category.clone();
        match category {
            None => {
                if let Some((category_name, (_, shop_items))) =
                    shop_config.shop_items.get_index(select_index as usize)
                {
                    menu_inventory.clear();

                    shop_state.selected_category = Some(category_name.clone());
                    for item in shop_items {
                        let next_slot = menu_inventory.first_empty_slot().unwrap();
                        let item_stack: ItemStack = item.offer.clone().into();
                        // Convert to team color
                        let item_stack = team_color.to_team_item_stack(item_stack);

                        menu_inventory.set_slot(next_slot, item_stack);
                    }

                    menu_inventory.set_slot(
                        SHOP_INVENTORY_TYPE.slot_count() as u16 - 1,
                        ItemStack::new(ItemKind::Barrier, 1, None),
                    );
                }
            }
            Some(category) => {
                // We are in a category window and are buying an item

                if select_index == SHOP_INVENTORY_TYPE.slot_count() as u16 - 1 {
                    // return to the main menu
                    shop_state.selected_category = None;
                    menu_inventory.clear();
                    for (idx, (_category_name, (category_item, _))) in
                        shop_config.shop_items.iter().enumerate()
                    {
                        // TODO: add category_name via nbt
                        menu_inventory.set_slot(idx as u16, category_item.clone());
                    }
                }

                if let Some((_, shop_items)) = shop_config.shop_items.get(&category) {
                    if let Some(item_to_buy) = shop_items.get(select_index as usize) {
                        let price = item_to_buy.price.clone().into();
                        let offer: ItemStack = item_to_buy.offer.clone().into();
                        // Convert to team color
                        let mut offer = team_color.to_team_item_stack(offer);

                        if let Some(ref mut nbt) = offer.nbt {
                            // Remove lore from item when bought
                            // TODO: its Display -> Lore
                            nbt.remove("Lore");
                        }

                        let mut bought = false;
                        if inventory.try_remove_all(&price) {
                            if !inventory.try_pickup_all(&offer) {
                                // refund
                                inventory.try_pickup_all(&price);
                            } else {
                                bought = true;
                            }
                        }

                        if bought {
                            client.play_sound(
                                Sound::BlockNoteBlockBell,
                                SoundCategory::Master,
                                position.0,
                                1.0,
                                1.8,
                            );
                        } else {
                            client.play_sound(
                                Sound::BlockNoteBlockBass,
                                SoundCategory::Master,
                                position.0,
                                1.0,
                                0.8,
                            );
                        }
                    }
                }

                // Buy an item
            }
        }

        inventory.set_changed();
    }
}
