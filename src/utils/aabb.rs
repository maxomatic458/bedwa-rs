use valence::{
    math::{Aabb, DVec3},
    ChunkLayer, Direction,
};

use super::ray_cast::aabb_full_block_intersections;

pub trait AabbExt {
    /// Calculate the center of the AABB
    /// # Returns
    /// DVec3 representing the center of the AABB
    fn center(&self) -> DVec3;
    /// Calculate the direction of the point from the center of the AABB
    fn projected_point_direction(&self, p: DVec3) -> Direction;
    /// Get the height of the AABB
    fn height(&self) -> f64;
    /// Get the width in the x direction
    fn width_x(&self) -> f64;
    /// Get the width in the z direction
    fn width_z(&self) -> f64;
    /// Move the aabb in a direction
    fn translate(&self, velocity: impl Into<DVec3>) -> Aabb;
    /// Move the aabb to another position
    fn translate_to(&self, position: DVec3) -> Aabb;
}

impl AabbExt for Aabb {
    fn center(&self) -> DVec3 {
        (self.min() + self.max()) / 2.0
    }

    fn projected_point_direction(&self, p: DVec3) -> Direction {
        let center = self.center();
        let diff = p - center;

        if diff.x.abs() > diff.y.abs() && diff.x.abs() > diff.z.abs() {
            if diff.x > 0.0 {
                Direction::East
            } else {
                Direction::West
            }
        } else if diff.y.abs() > diff.z.abs() {
            if diff.y > 0.0 {
                Direction::Up
            } else {
                Direction::Down
            }
        } else if diff.z > 0.0 {
            Direction::South
        } else {
            Direction::North
        }
    }

    fn height(&self) -> f64 {
        self.max().y - self.min().y
    }

    fn width_x(&self) -> f64 {
        self.max().x - self.min().x
    }

    fn width_z(&self) -> f64 {
        self.max().z - self.min().z
    }

    fn translate(&self, velocity: impl Into<DVec3>) -> Aabb {
        let velocity = velocity.into();
        Aabb::new(self.min() + velocity, self.max() + velocity)
    }

    fn translate_to(&self, position: DVec3) -> Aabb {
        Aabb::new(
            position,
            position + DVec3::new(self.width_x(), self.height(), self.width_z()),
        )
    }
}

pub fn is_on_ground(hitbox: &Aabb, layer: &ChunkLayer) -> bool {
    let hitbox = Aabb::new(hitbox.min() + DVec3::new(0.0, -0.001, 0.0), hitbox.max());

    let blocks_below = aabb_full_block_intersections(&hitbox);

    blocks_below.iter().any(|b| {
        if let Some(block) = layer.block(*b) {
            !block.state.is_air()
        } else {
            false
        }
    })
}
