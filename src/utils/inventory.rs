use valence::{prelude::Inventory, ItemStack};

pub trait InventoryExt {
    /// Clear the inventory
    fn clear(&mut self);
    /// try to pickup a stack of items
    /// # Returns
    /// Amount of items that were picked up
    fn try_pickup_stack(&mut self, stack: ItemStack) -> u8;
    /// Remove a stack of items from the inventory
    /// # Returns
    /// True if the stack was removed, false if the stack was not removed
    fn remove_stack(&mut self, stack: ItemStack) -> bool;
}

impl InventoryExt for Inventory {
    fn clear(&mut self) {
        for i in 0..self.slot_count() {
            self.set_slot(i, ItemStack::EMPTY);
        }
    }
    /// try to pickup a stack of items
    /// # Returns
    /// Amount of items that were picked up
    fn try_pickup_stack(&mut self, stack: ItemStack) -> u8 {
        let mut remaining = stack.count;

        let mergable: Vec<u16> = self
            .slots()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else if s.item == stack.item && s.count < 64 {
                    Some(i as u16)
                } else {
                    None
                }
            })
            .collect();

        for i in mergable {
            if remaining <= 0 {
                return stack.count as u8;
            }

            let slot_count = self.slot(i).count;
            let space = 64 - slot_count;
            let to_merge = remaining.min(space);

            self.set_slot(
                i,
                ItemStack::new(stack.item, slot_count + to_merge, stack.nbt.clone()),
            );

            remaining -= to_merge;
        }

        const HOTBAR_RANGE: std::ops::Range<u16> = 36..45;
        const INVENTORY_RANGE: std::ops::Range<u16> = 9..36;

        // first we try to fill up the hotbar
        while remaining > 0 {
            if let Some(next_empty) = self.first_empty_slot_in(HOTBAR_RANGE) {
                let to_fill = remaining.min(64);
                self.set_slot(
                    next_empty,
                    ItemStack::new(stack.item, to_fill, stack.nbt.clone()),
                );
                remaining -= to_fill;
            } else {
                break;
            }
        }

        // then we try to fill up the inventory
        while remaining > 0 {
            if let Some(next_empty) = self.first_empty_slot_in(INVENTORY_RANGE) {
                let to_fill = remaining.min(64);
                self.set_slot(
                    next_empty,
                    ItemStack::new(stack.item, to_fill, stack.nbt.clone()),
                );
                remaining -= to_fill;
            } else {
                break;
            }
        }

        (stack.count - remaining) as u8
    }

    fn remove_stack(&mut self, stack: ItemStack) -> bool {
        todo!()
    }
}
