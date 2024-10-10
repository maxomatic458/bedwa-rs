use valence::math::{Aabb, DVec3};

pub trait AabbExt {
    /// Calculate the center of the AABB
    /// # Returns
    /// DVec3 representing the center of the AABB
    fn center(&self) -> DVec3;
    /// Calculate a projected point that is exactly on the AABB boundary
    fn projected_point_exact(&self, p: DVec3) -> DVec3;
}

impl AabbExt for Aabb {
    fn center(&self) -> DVec3 {
        (self.min() + self.max()) / 2.0
    }

    fn projected_point_exact(&self, p: DVec3) -> DVec3 {
        let approx = self.projected_point(p);

        let mut result = DVec3::ZERO;
        let mut closest_idx = 0;

        for i in 0..3 {
            let diff = (approx[i] - p[i]).abs();
            if diff > result[closest_idx] {
                result[closest_idx] = diff;
                closest_idx = i;
            }
        }

        result[closest_idx] = self.min()[closest_idx];
        result
    }
}
