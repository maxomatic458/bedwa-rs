use bevy_ecs::entity::Entity;
use valence::{
    math::{Aabb, DVec3, Vec3},
    BlockPos, Direction,
};

/// Returns a list of all the blocks that are inside (or intersect) the given AABB
pub fn aabb_full_block_intersections(aabb: &Aabb) -> Vec<BlockPos> {
    let mut blocks = Vec::new();

    let min = aabb.min().floor();
    let max = aabb.max().ceil();

    for x in (min.x as i32)..(max.x as i32) {
        for y in (min.y as i32)..(max.y as i32) {
            for z in (min.z as i32)..(max.z as i32) {
                blocks.push(BlockPos { x, y, z });
            }
        }
    }

    blocks
}

#[derive(Debug)]
pub enum CollisionObject {
    Block { pos: DVec3, face_normal: Direction },
    Entity(Entity),
}

pub enum CollisionType {
    Block,
    Entity,
    Both,
}

/// Perform swept AABB collision
/// # Arguments
/// * `hb1` - The first moving AABB
/// * `velocity` - The velocity of the first AABB
/// * `hb2` - The second static AABB
/// # Returns
/// Entry time and x, y, z normal of the collision

type CollisionResult = (f64, (Option<i8>, Option<i8>, Option<i8>));

pub fn collide(hb1: &Aabb, velocity: Vec3, hb2: &Aabb) -> CollisionResult {
    let (vx, vy, vz) = (velocity.x, velocity.y, velocity.z);

    let no_collision = (1.0, (None, None, None));

    fn time(x: f64, y: f32) -> f64 {
        if y != 0.0 {
            x / y as f64
        } else if x > 0.0 {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        }
    }

    let x_entry = if vx != 0.0 {
        time(
            if vx > 0.0 {
                hb2.min().x - hb1.max().x
            } else {
                hb2.max().x - hb1.min().x
            },
            vx,
        )
    } else if hb1.max().x < hb2.min().x || hb1.min().x > hb2.max().x {
        return no_collision;
    } else {
        f64::NEG_INFINITY
    };

    let x_exit = if vx != 0.0 {
        time(
            if vx > 0.0 {
                hb2.max().x - hb1.min().x
            } else {
                hb2.min().x - hb1.max().x
            },
            vx,
        )
    } else {
        f64::INFINITY
    };

    let y_entry = if vy != 0.0 {
        time(
            if vy > 0.0 {
                hb2.min().y - hb1.max().y
            } else {
                hb2.max().y - hb1.min().y
            },
            vy,
        )
    } else if hb1.max().y < hb2.min().y || hb1.min().y > hb2.max().y {
        return no_collision;
    } else {
        f64::NEG_INFINITY
    };

    let y_exit = if vy != 0.0 {
        time(
            if vy > 0.0 {
                hb2.max().y - hb1.min().y
            } else {
                hb2.min().y - hb1.max().y
            },
            vy,
        )
    } else {
        f64::INFINITY
    };

    let z_entry = if vz != 0.0 {
        time(
            if vz > 0.0 {
                hb2.min().z - hb1.max().z
            } else {
                hb2.max().z - hb1.min().z
            },
            vz,
        )
    } else if hb1.max().z < hb2.min().z || hb1.min().z > hb2.max().z {
        return no_collision;
    } else {
        f64::NEG_INFINITY
    };

    let z_exit = if vz != 0.0 {
        time(
            if vz > 0.0 {
                hb2.max().z - hb1.min().z
            } else {
                hb2.min().z - hb1.max().z
            },
            vz,
        )
    } else {
        f64::INFINITY
    };

    if x_entry < 0.0 && y_entry < 0.0 && z_entry < 0.0 {
        return no_collision;
    }

    if x_entry > 1.0 || y_entry > 1.0 || z_entry > 1.0 {
        return no_collision;
    }

    let entry = x_entry.max(y_entry).max(z_entry);
    let exit = x_exit.min(y_exit).min(z_exit);

    if entry > exit {
        return no_collision;
    }

    let nx = if entry == x_entry {
        Some(if vx > 0.0 { -1 } else { 1 })
    } else {
        None
    };
    let ny = if entry == y_entry {
        Some(if vy > 0.0 { -1 } else { 1 })
    } else {
        None
    };
    let nz = if entry == z_entry {
        Some(if vz > 0.0 { -1 } else { 1 })
    } else {
        None
    };

    // Return the entry time and the normal of the surface collided with
    (entry, (nx, ny, nz))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_block_aabb_intersections1() {
        let aabb = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 1.0));
        let blocks = aabb_full_block_intersections(&aabb);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], BlockPos::new(0, 0, 0));
    }

    #[test]
    fn test_block_aabb_intersections2() {
        let aabb = Aabb::new(DVec3::new(0.5, 0.5, 0.5), DVec3::new(1.5, 1.5, 1.5));
        let blocks = aabb_full_block_intersections(&aabb);

        assert_eq!(blocks.len(), 8);
        assert_eq!(blocks[0], BlockPos::new(0, 0, 0));
        assert_eq!(blocks[1], BlockPos::new(0, 0, 1));
        assert_eq!(blocks[2], BlockPos::new(0, 1, 0));
        assert_eq!(blocks[3], BlockPos::new(0, 1, 1));
        assert_eq!(blocks[4], BlockPos::new(1, 0, 0));
        assert_eq!(blocks[5], BlockPos::new(1, 0, 1));
        assert_eq!(blocks[6], BlockPos::new(1, 1, 0));
        assert_eq!(blocks[7], BlockPos::new(1, 1, 1));
    }
}
