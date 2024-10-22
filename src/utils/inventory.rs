use valence::{prelude::Inventory, ItemStack};

pub trait InventoryExt {
    /// Clear the inventory
    fn clear(&mut self);

    /// Returns a tuple of inventory indices and how many items of the stack
    /// could fit in them.
    /// It is not guaranteed that the stack fully fits in the inventory.
    /// This tries to mirror vanilla behavior.
    fn fill_slots(&self, stack: &ItemStack) -> Vec<(u16, u8)>;

    /// Returns a tuple of inventory indices and how many items of that slot
    /// could be removed.
    /// It is not guaranteed that the full stack can be removed from the
    /// inventory.
    fn remove_slots(&self, stack: &ItemStack) -> Vec<(u16, u8)>;

    /// Try to pickup a stack of items,
    /// It is not guaranteed that the whole stack will
    /// go in the inventory
    /// # Returns
    /// The amount of itemrs that were picked up
    fn try_pickup_stack(&mut self, stack: &ItemStack) -> u8;

    /// try to pickup a stack of items, but only pick it up
    /// if everything fits in the inventory, if not return false
    fn try_pickup_all(&mut self, stack: &ItemStack) -> bool;

    /// Remove a stack of items from the inventory
    /// # Returns
    /// True if the stack was removed, false if the stack was not removed
    fn try_remove_all(&mut self, stack: &ItemStack) -> bool;

    /// Checks if the inventory contains at least the one of the given stack
    fn check_contains_stack(&self, stack: &ItemStack, ignore_nbt: bool) -> bool;
}

impl InventoryExt for Inventory {
    fn clear(&mut self) {
        for i in 0..self.slot_count() {
            self.set_slot(i, ItemStack::EMPTY);
        }
    }

    fn fill_slots(&self, stack: &ItemStack) -> Vec<(u16, u8)> {
        let mut slot_fills = vec![];
        let mut remaining = stack.count;

        let mergeable: Vec<u16> = self
            .slots()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else if s.item == stack.item && s.count < s.item.max_stack() {
                    Some(i as u16)
                } else {
                    None
                }
            })
            .collect();

        for i in mergeable {
            if remaining <= 0 {
                break;
            }

            let slot_count = self.slot(i).count;
            let space = stack.item.max_stack() - slot_count;
            let to_merge = remaining.min(space);

            slot_fills.push((i, to_merge as u8));
            remaining -= to_merge;
        }

        const HOTBAR_RANGE: std::ops::Range<u16> = 36..45;
        const INVENTORY_RANGE: std::ops::Range<u16> = 9..36;

        // first we try to fill up the hotbar
        while remaining > 0 {
            if let Some(next_empty) = self.first_empty_slot_in(HOTBAR_RANGE) {
                let to_fill = remaining.min(stack.item.max_stack());

                slot_fills.push((next_empty, to_fill as u8));

                remaining -= to_fill;
            } else {
                break;
            }
        }

        // then we try to fill up the inventory
        while remaining > 0 {
            if let Some(next_empty) = self.first_empty_slot_in(INVENTORY_RANGE) {
                let to_fill = remaining.min(stack.item.max_stack());
                slot_fills.push((next_empty, to_fill as u8));
                remaining -= to_fill;
            } else {
                break;
            }
        }

        slot_fills
    }

    fn try_pickup_stack(&mut self, stack: &ItemStack) -> u8 {
        let mut picked_up = 0;
        let slots_to_fill = self.fill_slots(stack);
        for (slot, to_fill) in slots_to_fill {
            let current_stack = self.slot(slot).clone();
            picked_up += to_fill;
            let new_stack = ItemStack::new(
                stack.item,
                current_stack.count + to_fill as i8,
                stack.nbt.clone(),
            );
            self.set_slot(slot, new_stack);
        }

        picked_up
    }

    fn try_pickup_all(&mut self, stack: &ItemStack) -> bool {
        let slots_to_fill = self.fill_slots(stack);

        let filled: u8 = slots_to_fill.iter().map(|(_, c)| c).sum();
        if filled != stack.count as u8 {
            return false;
        }

        self.try_pickup_stack(stack);
        true
    }

    fn remove_slots(&self, stack: &ItemStack) -> Vec<(u16, u8)> {
        let mut slot_removes = vec![];
        let mut to_remove = stack.count;

        let removable: Vec<u16> = self
            .slots()
            .enumerate()
            .rev()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else if s.item == stack.item {
                    Some(i as u16)
                } else {
                    None
                }
            })
            .collect();

        for i in removable {
            if to_remove <= 0 {
                break;
            }
            let slot = self.slot(i);
            let can_remove = to_remove.min(slot.count);

            to_remove -= can_remove;

            slot_removes.push((i, can_remove as u8));
        }

        slot_removes
    }

    fn try_remove_all(&mut self, stack: &ItemStack) -> bool {
        let slots_to_remove = self.remove_slots(stack);

        let removed: u8 = slots_to_remove.iter().map(|(_, r)| r).sum();
        if removed != stack.count as u8 {
            return false;
        }

        for (idx, remove) in slots_to_remove {
            let old_stack = self.slot(idx);

            let new_stack = if old_stack.count as u8 == remove {
                ItemStack::EMPTY
            } else {
                ItemStack::new(
                    old_stack.item,
                    old_stack.count - remove as i8,
                    old_stack.nbt.clone(),
                )
            };

            self.set_slot(idx, new_stack);
        }

        true
    }

    fn check_contains_stack(&self, stack: &ItemStack, ignore_nbt: bool) -> bool {
        self.slots().any(|s| {
            s.item == stack.item && s.count >= stack.count && {
                if ignore_nbt {
                    true
                } else {
                    s.nbt == stack.nbt
                }
            }
        })
    }
}
