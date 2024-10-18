use crate::colors::TeamColor;
use ordermap::{OrderMap, OrderSet};
use serde::{Deserialize, Serialize};
use valence::math::DVec3;
use valence::nbt::Compound;
use valence::prelude::Resource;
use valence::{ItemKind, ItemStack};

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct ConfigVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ConfigVec3 {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn up() -> Self {
        Self::new(0, 1, 0)
    }
}

impl std::fmt::Display for ConfigVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<ConfigVec3> for DVec3 {
    fn from(val: ConfigVec3) -> Self {
        DVec3::new(val.x as f64, val.y as f64, val.z as f64)
    }
}

impl std::ops::Add for ConfigVec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// impl Vec3 {
//     // pub fn from_valence_vec3(vec3: valence::command::parsers::Vec3)
// }

#[derive(Debug, Serialize, Deserialize, Resource, Clone)]
pub struct BedwarsConfig {
    /// Bounds of the bedwars arena
    pub bounds: (ConfigVec3, ConfigVec3),
    /// Team name -> team color
    pub teams: OrderMap<String, TeamColor>,
    /// Team name -> spawn point
    pub spawns: OrderMap<String, ConfigVec3>,
    /// Team name -> bed location
    pub beds: OrderMap<String, OrderSet<ConfigVec3>>,
    /// Shop location, yaw ->? team name
    pub shops: Vec<((ConfigVec3, f32), Option<String>)>,
    /// Resource spawner location -> (resource type, spawn interval secs, team name)
    pub resource_spawners: Vec<(ConfigVec3, SerItemStack, f32, Option<String>)>,
    /// Lobby spawn point
    pub lobby_spawn: ConfigVec3,
    /// Spectator spawn point
    pub spectator_spawn: ConfigVec3,
}

/// Represents a WIP bedwars config, which will be changed
/// and can be saved once every value is set
#[derive(Debug, Serialize, Deserialize, Default, Resource)]
pub struct BedwarsWIPConfig {
    /// Bounds of the bedwars arena
    pub bounds: Option<(ConfigVec3, ConfigVec3)>,
    /// Team name -> team color
    pub teams: OrderMap<String, TeamColor>,
    /// Team name -> spawn point
    pub spawns: OrderMap<String, ConfigVec3>,
    /// Team name -> bed location
    pub beds: OrderMap<String, OrderSet<ConfigVec3>>,
    /// Shop location, yaw ->? team name
    pub shops: Vec<((ConfigVec3, f32), Option<String>)>,
    /// Resource spawner location -> (resource type, spawn interval secs, team name)
    pub resource_spawners: Vec<(ConfigVec3, SerItemStack, f32, Option<String>)>,
    /// Lobby spawn point
    pub lobby_spawn: Option<ConfigVec3>,
    /// Spectator spawn point
    pub spectator_spawn: Option<ConfigVec3>,
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
#[derive(Debug, Clone, PartialEq)]
struct SerItemKind(pub ItemKind);

impl Serialize for SerItemKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerItemKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self(ItemKind::from_str(&s).unwrap()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SerItemStack {
    item: SerItemKind,
    count: i8,
    nbt: Option<Compound>,
}

impl From<SerItemStack> for ItemStack {
    fn from(stack: SerItemStack) -> Self {
        ItemStack::new(stack.item.0, stack.count, stack.nbt)
    }
}

impl From<ItemStack> for SerItemStack {
    fn from(stack: ItemStack) -> Self {
        Self {
            item: SerItemKind(stack.item),
            count: stack.count,
            nbt: stack.nbt,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ShopPrice {
    item_id: String,
    stack_size: i8,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShopOffer {
    pub offer: SerItemStack,
    pub price: SerItemStack,
}

#[derive(Debug, Serialize, Deserialize, Clone, Resource)]
pub struct ShopConfig {
    /// Category -> (item being sold, price)
    pub shop_items: OrderMap<String, (SerItemStack, Vec<ShopOffer>)>,
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
    // todo!();
}
