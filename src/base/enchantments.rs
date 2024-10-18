pub fn sharpness_extra_dmg(level: u32) -> f32 {
    level as f32 * 0.5 + 0.5
}

pub fn knockback_extra_range(level: u32) -> f32 {
    level as f32 * 3.0
}
