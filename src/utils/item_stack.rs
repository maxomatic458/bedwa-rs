use valence::ItemStack;

use crate::base::enchantments::{
    fire_aspect_burn_time, flame_burn_time, knockback_extra_range, sharpness_extra_dmg,
    Enchantment, ItemStackExtEnchantments,
};

use super::item_kind::ItemKindExtWeapons;

pub trait ItemStackExtWeapons {
    /// Get the damage the item does, including enchantments.
    fn damage(&self) -> f32;
    /// Get the knockback range of the item, including enchantments.
    fn knockback_extra(&self) -> f32;

    fn burn_time(&self) -> f32;
}

impl ItemStackExtWeapons for ItemStack {
    fn damage(&self) -> f32 {
        let base = self.item.damage();

        if let Some(sharpness_lvl) = self.enchantments().get(&Enchantment::Sharpness) {
            base + sharpness_extra_dmg(*sharpness_lvl)
        } else {
            base
        }
    }

    fn knockback_extra(&self) -> f32 {
        if let Some(knockback_lvl) = self.enchantments().get(&Enchantment::Knockback) {
            knockback_extra_range(*knockback_lvl)
        } else {
            0.0
        }
    }

    fn burn_time(&self) -> f32 {
        if let Some(fire_aspect_lvl) = self.enchantments().get(&Enchantment::FireAspect) {
            fire_aspect_burn_time(*fire_aspect_lvl)
        } else if let Some(flame_lvl) = self.enchantments().get(&Enchantment::Flame) {
            flame_burn_time(*flame_lvl)
        } else {
            0.0
        }
    }
}
