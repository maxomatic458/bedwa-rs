use std::collections::HashMap;

use valence::{
    nbt::{value::ValueRef, Value},
    ItemStack,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Enchantment {
    Sharpness,
    Knockback,
    Protection,
}

impl Enchantment {
    fn from_str(string_id: &str) -> Option<Enchantment> {
        match string_id {
            "minecraft:sharpness" => Some(Enchantment::Sharpness),
            "minecraft:knockback" => Some(Enchantment::Knockback),
            "minecraft:protection" => Some(Enchantment::Protection),
            _ => None,
        }
    }
}

/// Calculates the extra damage given by the sharpness enchantment.
pub fn sharpness_extra_dmg(level: u32) -> f32 {
    tracing::error!("level: {}", level);
    if level == 0 {
        return 0.0;
    }
    level as f32 * 0.5 + 0.5
}

/// Calculates the extra knockback range given by the knockback enchantment.
pub fn knockback_extra_range(level: u32) -> f32 {
    level as f32 * 3.0
}

/// Calculates the damage reduction given by the protection enchantment.
pub fn protection_reduction(level: u32) -> f32 {
    level as f32 * 0.04
}

pub trait ItemStackExtEnchantments {
    /// Get the enchantments of an item stack via NBT.
    fn enchantments(&self) -> HashMap<Enchantment, u32>;
}

impl ItemStackExtEnchantments for ItemStack {
    fn enchantments(&self) -> HashMap<Enchantment, u32> {
        let mut enchantments = HashMap::new();
        if let Some(nbt) = &self.nbt {
            if let Some(Value::List(enchants)) = nbt.get("Enchantments") {
                for enchant in enchants {
                    if let ValueRef::Compound(enchant) = enchant {
                        if let (Some(Value::String(id)), Some(Value::Long(level))) =
                            (enchant.get("id"), enchant.get("lvl"))
                        {
                            if let Some(enchantment) = Enchantment::from_str(id) {
                                enchantments.insert(enchantment, *level as u32);
                            }
                        }
                    }
                }
            }
        }

        enchantments
    }
}
