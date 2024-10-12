use std::iter::Sum;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::{With, Without},
    system::{Commands, Query, Res},
};
use bevy_state::state::OnEnter;
use valence::{
    app::{Plugin, Update},
    entity::{player::PlayerEntity, EntityLayerId},
    prelude::{Component, InteractEntityEvent, Inventory, InventoryKind},
    ChunkLayer, EntityLayer,
};

use crate::{
    base::death::IsDead,
    bedwars_config::{BedwarsConfig, ShopConfig, ShopOffer},
    menu::{ItemMenu, MenuItemSelect},
    utils::inventory,
    GameState, Team,
};

// const SHOP_ENTITY_BUNDLE =

#[derive(Debug, Clone, Component)]
pub struct Shop;

#[derive(Component, Default)]
pub struct ShopState {
    selected_catagory: Option<String>,
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut valence::prelude::App) {
        app.add_systems(OnEnter(GameState::Match), (init_shops,))
            .add_systems(Update, (on_click_shopitem, on_shop_open));
    }
}

/// Initialize the shops
fn init_shops(
    mut commands: Commands,
    bedwars_config: Res<BedwarsConfig>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    tracing::error!("called");

    let layer = layers.single();
    for (pos, team) in &bedwars_config.shops {
        let mut entity_commands = commands.spawn(valence::entity::villager::VillagerEntityBundle {
            layer: EntityLayerId(layer),
            position: valence::entity::Position(pos.clone().into()),
            ..Default::default()
        });

        entity_commands.insert(Shop);

        tracing::info!("Initialized shop at {:?}", pos);
        if let Some(team) = team {
            entity_commands.insert(Team(team.clone()));
        }
    }
}

/// The player opens the shop by left or right clicking the villager
fn on_shop_open(
    mut commands: Commands,
    mut events: EventReader<InteractEntityEvent>,
    players: Query<Entity, (With<PlayerEntity>, Without<IsDead>)>,
    shops: Query<&Shop>,
    shop_config: Res<ShopConfig>,
) {
    for event in events.read() {
        let Ok(player) = players.get(event.client) else {
            continue;
        };
        let Ok(shop) = shops.get(event.entity) else {
            continue;
        };

        let shop_menu = menu_from_shop_config(&shop_config);

        commands
            .entity(player)
            .insert(shop_menu)
            .insert(ShopState::default());
    }
}

fn menu_from_shop_config(shop_config: &ShopConfig) -> ItemMenu {
    let mut shop_menu = Inventory::new(InventoryKind::Generic9x5);
    for (idx, (category_name, (category_item, items))) in shop_config.shop_items.iter().enumerate()
    {
        shop_menu.set_slot(idx as u16, category_item.clone());
    }

    ItemMenu::new(shop_menu)
}

fn on_click_shopitem(
    mut clients: Query<(Entity, &mut Inventory, &mut ShopState)>,
    mut commands: Commands,
    mut events: EventReader<MenuItemSelect>,
    shop_config: Res<ShopConfig>,
) {
    for event in events.read() {
        let select_index = event.idx;
        let Ok((player_ent, mut inventory, mut shop_state)) = clients.get_mut(event.client) else {
            continue;
        };
        if let Some((shop_catagory_name, (_, shop_items))) =
            shop_config.shop_items.get_index(select_index.into())
        {
            commands.entity(player_ent).remove::<ItemMenu>();

            shop_state.selected_catagory = Some(shop_catagory_name.clone());
            let mut inv = Inventory::new(InventoryKind::Generic9x5);
            for item in shop_items {
                let next_slot = inv.first_empty_slot().unwrap();
                let item_stack = item.offer.clone();
                inv.set_slot(next_slot, item_stack);
            }
            let menu = ItemMenu::new(inv);
            commands.entity(player_ent).insert(menu);
        }
    }
}
