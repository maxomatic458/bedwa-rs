use crate::colors::TeamColor;
use ordermap::OrderMap;
use serde::{Deserialize, Serialize};
use valence::math::DVec3;
use valence::nbt::Compound;
use valence::prelude::{Block, Resource};
use valence::{BlockState, ItemKind, ItemStack};

pub const SHOP_CONFIG_NAME: &str = "shop.json";
pub const WORLD_CONFIG_NAME: &str = "bw-world.json";

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

#[derive(Debug, Serialize, Deserialize, Resource, Clone)]
pub struct WorldConfig {
    /// Bounds of the bedwars arena
    pub bounds: (ConfigVec3, ConfigVec3),
    /// Team name -> team color
    pub teams: OrderMap<String, TeamColor>,
    /// Team name -> spawn point
    pub spawns: OrderMap<String, ConfigVec3>,
    /// Team name -> bed location & block
    pub beds: OrderMap<String, Vec<(ConfigVec3, SerBlock)>>,
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
pub struct WIPWorldConfig {
    /// Bounds of the bedwars arena
    pub bounds: Option<(ConfigVec3, ConfigVec3)>,
    /// Team name -> team color
    pub teams: OrderMap<String, TeamColor>,
    /// Team name -> spawn point
    pub spawns: OrderMap<String, ConfigVec3>,
    /// Team name -> bed location & block
    pub beds: OrderMap<String, Vec<(ConfigVec3, SerBlock)>>,
    /// Shop location, yaw ->? team name
    pub shops: Vec<((ConfigVec3, f32), Option<String>)>,
    /// Resource spawner location -> (resource type, spawn interval secs, team name)
    pub resource_spawners: Vec<(ConfigVec3, SerItemStack, f32, Option<String>)>,
    /// Lobby spawn point
    pub lobby_spawn: Option<ConfigVec3>,
    /// Spectator spawn point
    pub spectator_spawn: Option<ConfigVec3>,
}

impl WIPWorldConfig {
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

    pub fn from_saved_config(config: &WorldConfig) -> Self {
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
#[derive(Debug, Clone, PartialEq)]
pub struct SerBlockState(BlockState);

impl serde::Serialize for SerBlockState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_raw().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SerBlockState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = u16::deserialize(deserializer)?;
        Ok(Self(BlockState::from_raw(s).unwrap()))
    }
}

impl From<SerBlockState> for BlockState {
    fn from(state: SerBlockState) -> Self {
        state.0
    }
}

impl From<BlockState> for SerBlockState {
    fn from(state: BlockState) -> Self {
        Self(state)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SerBlock {
    pub state: SerBlockState,
    pub nbt: Option<Compound>,
}

impl From<SerBlock> for Block {
    fn from(block: SerBlock) -> Self {
        Block::new(block.state.into(), block.nbt)
    }
}

impl From<Block> for SerBlock {
    fn from(block: Block) -> Self {
        Self {
            state: SerBlockState::from(block.state),
            nbt: block.nbt,
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
    pub shop_items: OrderMap<String, (SerItemStack, Vec<ShopOffer>)>,
}

pub fn load_config() -> color_eyre::Result<WorldConfig> {
    let config = std::fs::read_to_string(WORLD_CONFIG_NAME)?;
    let config: WorldConfig = serde_json::from_str(&config)?;
    Ok(config)
}

pub fn load_trader_config() -> color_eyre::Result<ShopConfig> {
    let config = std::fs::read_to_string(SHOP_CONFIG_NAME)?;
    let config: ShopConfig = serde_json::from_str(&config)?;
    Ok(config)
}
