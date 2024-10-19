use crate::utils::enchantments::{protection_reduction, Enchantment, ItemStackExtEnchantments};
use valence::{prelude::Equipment, ItemKind, ItemStack};

pub trait ItemKindExtArmor {
    /// The armor points of that item
    fn armor_points(&self) -> f32;
    /// The toughness of that item
    fn toughness(&self) -> f32;
}

impl ItemKindExtArmor for ItemKind {
    fn armor_points(&self) -> f32 {
        match self {
            ItemKind::LeatherHelmet => 1.0,
            ItemKind::LeatherChestplate => 3.0,
            ItemKind::LeatherLeggings => 2.0,
            ItemKind::LeatherBoots => 1.0,

            ItemKind::ChainmailHelmet => 2.0,
            ItemKind::ChainmailChestplate => 5.0,
            ItemKind::ChainmailLeggings => 4.0,
            ItemKind::ChainmailBoots => 1.0,

            ItemKind::IronHelmet => 2.0,
            ItemKind::IronChestplate => 6.0,
            ItemKind::IronLeggings => 5.0,
            ItemKind::IronBoots => 2.0,

            ItemKind::GoldenHelmet => 2.0,
            ItemKind::GoldenChestplate => 5.0,
            ItemKind::GoldenLeggings => 3.0,
            ItemKind::GoldenBoots => 1.0,

            ItemKind::DiamondHelmet => 3.0,
            ItemKind::DiamondChestplate => 8.0,
            ItemKind::DiamondLeggings => 6.0,
            ItemKind::DiamondBoots => 3.0,

            ItemKind::NetheriteHelmet => 3.0,
            ItemKind::NetheriteChestplate => 8.0,
            ItemKind::NetheriteLeggings => 6.0,
            ItemKind::NetheriteBoots => 3.0,
            _ => 0.0,
        }
    }

    fn toughness(&self) -> f32 {
        match self {
            ItemKind::DiamondHelmet => 2.0,
            ItemKind::DiamondChestplate => 2.0,
            ItemKind::DiamondLeggings => 2.0,
            ItemKind::DiamondBoots => 2.0,

            ItemKind::NetheriteHelmet => 3.0,
            ItemKind::NetheriteChestplate => 3.0,
            ItemKind::NetheriteLeggings => 3.0,
            ItemKind::NetheriteBoots => 3.0,
            _ => 0.0,
        }
    }
}

/// Calculates the final damage
fn calculate_damage_armor(damage: f32, armor_points: f32, toughness: f32) -> f32 {
    // damage after armor points
    let damage = damage
        * (1.0
            - (20.0_f32.min(
                ((armor_points) / 5.0).max(armor_points - (4.0 * damage / (toughness + 8.0))),
            ) / 25.0));

    damage.max(0.0)
}

pub trait EquipmentExtDamageReduction {
    /// Calculate the real damage the player will receive after
    /// accounting for armor points, toughness, and enchantments.
    fn received_damage(&self, damage: f32) -> f32;
    /// Get the armor points of the equipment
    fn armor_points(&self) -> f32;
    /// Get the toughness of the equipment
    fn toughness(&self) -> f32;
    /// Get the reduction of protection enchantments
    fn protection_reduction(&self) -> f32;
}

impl EquipmentExtDamageReduction for Equipment {
    fn received_damage(&self, damage: f32) -> f32 {
        let armor_points = self.armor_points();
        let toughness = self.toughness();

        let after_armor = calculate_damage_armor(damage, armor_points, toughness);
        let protection_reduction = self.protection_reduction();

        after_armor * (1.0 - protection_reduction)
    }

    fn armor_points(&self) -> f32 {
        self.head().item.armor_points()
            + self.chest().item.armor_points()
            + self.legs().item.armor_points()
            + self.feet().item.armor_points()
    }

    fn toughness(&self) -> f32 {
        self.head().item.toughness()
            + self.chest().item.toughness()
            + self.legs().item.toughness()
            + self.feet().item.toughness()
    }

    fn protection_reduction(&self) -> f32 {
        self.head().protection_reduction()
            + self.chest().protection_reduction()
            + self.legs().protection_reduction()
            + self.feet().protection_reduction()
    }
}

pub trait ItemStackExtArmor {
    /// Get the damage reduction (in %) caused by the protection enchantment.
    fn protection_reduction(&self) -> f32;
}

impl ItemStackExtArmor for ItemStack {
    fn protection_reduction(&self) -> f32 {
        if let Some(protection_lvl) = self.enchantments().get(&Enchantment::Protection) {
            protection_reduction(*protection_lvl)
        } else {
            0.0
        }
    }
}
