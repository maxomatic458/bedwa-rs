use crate::colors::TeamColor;
use ordermap::OrderMap;
use serde::{Deserialize, Serialize};
use valence::math::DVec3;
use valence::nbt::Compound;
use valence::prelude::Resource;
use valence::{ItemKind, ItemStack};

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Vec3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Vec3> for DVec3 {
    fn from(val: Vec3) -> Self {
        DVec3::new(val.x as f64, val.y as f64, val.z as f64)
    }
}

// impl Vec3 {
//     // pub fn from_valence_vec3(vec3: valence::command::parsers::Vec3)
// }

#[derive(Debug, Serialize, Deserialize, Resource, Clone)]
pub struct BedwarsConfig {
    /// Bounds of the bedwars arena
    pub bounds: (Vec3, Vec3),
    /// Team name -> team color
    pub teams: OrderMap<String, TeamColor>,
    /// Team name -> spawn point
    pub spawns: OrderMap<String, Vec3>,
    /// Team name -> bed location
    pub beds: OrderMap<String, Vec3>,
    /// Shop location ->? team name
    pub shops: Vec<(Vec3, Option<String>)>,
    /// Resource spawner location -> (resource type, team name)
    pub resource_spawners: Vec<(Vec3, (String, Option<String>))>,
    /// Lobby spawn point
    pub lobby_spawn: Vec3,
    /// Spectator spawn point
    pub spectator_spawn: Vec3,
}

/// Represents a WIP bedwars config, which will be changed
/// and can be saved once every value is set
#[derive(Debug, Serialize, Deserialize, Default, Resource)]
pub struct BedwarsWIPConfig {
    /// Bounds of the bedwars arena
    pub bounds: Option<(Vec3, Vec3)>,
    /// Team name -> team color
    pub teams: OrderMap<String, TeamColor>,
    /// Team name -> spawn point
    pub spawns: OrderMap<String, Vec3>,
    /// Team name -> bed location
    pub beds: OrderMap<String, Vec3>,
    /// Shop location ->? team name
    pub shops: Vec<(Vec3, Option<String>)>,
    /// Resource spawner location -> (resource type, team name)
    pub resource_spawners: Vec<(Vec3, (String, Option<String>))>,
    /// Lobby spawn point
    pub lobby_spawn: Option<Vec3>,
    /// Spectator spawn point
    pub spectator_spawn: Option<Vec3>,
}

impl BedwarsWIPConfig {
    pub fn is_finished(&self) -> bool {
        self.bounds.is_some()
            && !self.teams.is_empty()
            && !self.spawns.is_empty()
            && !self.beds.is_empty()
            && !self.shops.is_empty()
            && !self.resource_spawners.is_empty()
            && self.lobby_spawn.is_some()
            && self.spectator_spawn.is_some()
    }

    pub fn from_saved_config(config: &BedwarsConfig) -> Self {
        Self {
            bounds: Some(config.bounds.clone()),
            teams: config.teams.clone(),
            spawns: config.spawns.clone(),
            beds: config.beds.clone(),
            shops: config.shops.clone(),
            resource_spawners: config.resource_spawners.clone(),
            lobby_spawn: Some(config.lobby_spawn.clone()),
            spectator_spawn: Some(config.spectator_spawn.clone()),
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// pub struct ShopItem {
//     item_id: String,
//     stack_size: i8,
//     nbt: Option<Compound>,
// }

// impl ShopItem {
//     pub fn to_item_stack(&self) -> ItemStack {
//         ItemStack::new(
//             ItemKind::from_str(&self.item_id).unwrap(),
//             self.stack_size,
//             self.nbt.clone(),
//         )
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// pub struct ShopPrice {
//     item_id: String,
//     stack_size: i8,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Resource)]
pub struct ShopConfig {
    /// Category -> (item being sold, price)
    pub shop_items: OrderMap<String, (ItemStack, Vec<(ItemStack, ItemStack)>)>,
}

pub fn load_config() -> color_eyre::Result<BedwarsConfig> {
    let config = std::fs::read_to_string("bedwars_config.json")?;
    let config: BedwarsConfig = serde_json::from_str(&config)?;
    Ok(config)
}

pub fn load_trader_config() -> color_eyre::Result<ShopConfig> {
    let config = std::fs::read_to_string("shop_config.json")?;
    let config: ShopConfig = serde_json::from_str(&config)?;
    Ok(config)
}
