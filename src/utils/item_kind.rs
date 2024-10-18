use valence::{ItemKind, ItemStack};

pub trait ItemStackExt {
    fn damage(&self) -> f32;
    fn knockback(&self) -> f32;
    fn burn_duration(&self) -> Option<u32>;
    fn use_ammo(&self) -> bool;
}

impl ItemStackExt for ItemStack {
    fn damage(&self) -> f32 {
        match self.item {
            ItemKind::WoodenSword => 4.0,
            ItemKind::WoodenPickaxe => 2.0,
            ItemKind::WoodenHoe => 1.0,
            ItemKind::WoodenShovel => 2.5,

            ItemKind::StoneSword => 5.0,
            ItemKind::StonePickaxe => 3.0,
            ItemKind::StoneHoe => 1.0,
            ItemKind::StoneShovel => 3.5,

            ItemKind::IronSword => 6.0,
            ItemKind::IronPickaxe => 5.0,
            ItemKind::IronHoe => 1.0,
            ItemKind::IronShovel => 2.5,

            ItemKind::GoldenSword => 4.0,
            ItemKind::GoldenPickaxe => 2.0,
            ItemKind::GoldenHoe => 1.0,
            ItemKind::GoldenShovel => 2.5,

            ItemKind::DiamondSword => 7.0,
            ItemKind::DiamondPickaxe => 5.0,
            ItemKind::DiamondHoe => 1.0,
            ItemKind::DiamondShovel => 5.5,

            ItemKind::NetheriteSword => 8.0,
            ItemKind::NetheritePickaxe => 6.0,
            ItemKind::NetheriteHoe => 1.0,
            ItemKind::NetheriteShovel => 6.5,
            _ => 0.5,
        }
    }

    fn knockback(&self) -> f32 {
        todo!()
    }

    fn burn_duration(&self) -> Option<u32> {
        todo!()
    }

    fn use_ammo(&self) -> bool {
        todo!()
    }
}
