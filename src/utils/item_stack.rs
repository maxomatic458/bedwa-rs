use valence::ItemStack;

use super::{
    enchantments::{sharpness_extra_dmg, Enchantment, ItemStackExtEnchantments},
    item_kind::ItemKindExtWeapons,
};

pub trait ItemStackExtWeapons {
    /// Get the damage the item does, including enchantments.
    fn damage(&self) -> f32;
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
}
