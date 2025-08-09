use cgmath::{BaseNum, EuclideanSpace, InnerSpace, Point3, Vector3, Zero};

use super::ray::Ray;
use crate::{math::ray::RayCollision, state::world::BlockPos};

#[derive(Debug)]
pub struct AABB<S: BaseNum> {
    pub start: Point3<S>,
    pub end: Point3<S>,
}

impl<S: BaseNum + PartialOrd> AABB<S> {
    pub fn new(p0: &Point3<S>, p1: &Point3<S>) -> Self {
        let mut start = *p0;
        let mut end = *p1;
        // Ensure start < end
        if end.x < start.x {
            std::mem::swap(&mut start.x, &mut end.x);
        }
        if end.y < start.y {
            std::mem::swap(&mut start.y, &mut end.y);
        }
        if end.z < start.z {
            std::mem::swap(&mut start.z, &mut end.z);
        }

        Self { start, end }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.start.x <= other.end.x
            && self.end.x >= other.start.x
            && self.start.y <= other.end.y
            && self.end.y >= other.start.y
            && self.start.z <= other.end.z
            && self.end.z >= other.start.z
    }

    pub fn intersects_point(&self, point: &Point3<S>) -> bool {
        [0, 1, 2]
            .into_iter()
            .all(|axis| self.start[axis] <= point[axis] && point[axis] <= self.end[axis])
    }

    pub fn to_f32(&self) -> AABB<f32> {
        // Cast here is fine as we either lose f64 precision, or go from integer
        AABB {
            start: self.start.cast().unwrap(),
            end: self.end.cast().unwrap(),
        }
    }

    /// Iterate over all of the vertices of this bounding box
    pub fn iter_points<'a>(&'a self) -> impl Iterator<Item = Point3<S>> + 'a {
        let points = [&self.start, &self.end];
        (0..8).map(move |i| {
            let x = (i >> 2) & 1;
            let y = (i >> 1) & 1;
            let z = i & 1;

            Point3::new(points[x].x, points[y].y, points[z].z)
        })
    }

    /// Expand this AABB by convolving the other one over it.
    pub fn minkowski_sum(&self, other: &Self) -> Self {
        Self {
            start: self.start + other.start.to_vec(),
            end: self.end + other.end.to_vec(),
        }
    }

    pub fn size(&self) -> Vector3<S> {
        self.end - self.start
    }

    pub fn translate(&self, offset: &Vector3<S>) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
}

impl AABB<f32> {
    /// Check if a ray intersects with this AABB. Returns the distance to the intersection point
    /// if it does, along with the surface normal at the intersection.
    /// https://en.wikipedia.org/wiki/Slab_method
    pub fn intersect_ray(&self, ray: &Ray) -> Option<RayCollision> {
        assert!(ray.direction.magnitude2() > 0.);
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;
        let mut hit_axis = 0;
        let mut hit_min_face = true;

        // Iterate over all 3 axes
        for i in 0..3 {
            if ray.direction[i].abs() < f32::EPSILON {
                // Ray is parallel to the slab
                if ray.pos[i] < self.start[i] || ray.pos[i] > self.end[i] {
                    return None; // Ray is outside the slab and parallel to it
                }
            } else {
                let inv_dir = 1.0 / ray.direction[i];
                let mut t1 = (self.start[i] - ray.pos[i]) * inv_dir;
                let mut t2 = (self.end[i] - ray.pos[i]) * inv_dir;

                let mut swapped = false;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                    swapped = true;
                }

                // Track which face we hit for normal calculation
                if t1 > tmin {
                    tmin = t1;
                    hit_axis = i;
                    hit_min_face = !swapped; // If we swapped, we hit the max face
                }

                tmax = tmax.min(t2);

                if tmin > tmax {
                    return None;
                }
            }
        }

        // Return the closest intersection point that's in front of the ray
        if tmax < 0. {
            return None;
        }

        let dist = if tmin >= 0.0 { tmin } else { tmax };

        // Calculate the normal vector
        // The normal points outward from the face that was hit
        let mut normal = Vector3::zero();
        if tmin >= 0.0 {
            // We hit the entry face
            normal[hit_axis] = if hit_min_face { -1.0 } else { 1.0 };
        } else {
            // We hit the exit face (ray started inside the box)
            // Find which axis corresponds to tmax
            for i in 0..3 {
                if ray.direction[i].abs() >= f32::EPSILON {
                    let inv_dir = 1.0 / ray.direction[i];
                    let t1 = (self.start[i] - ray.pos[i]) * inv_dir;
                    let t2 = (self.end[i] - ray.pos[i]) * inv_dir;
                    let tmax_candidate = t1.max(t2);

                    if (tmax_candidate - tmax).abs() < f32::EPSILON {
                        normal[i] = if t2 > t1 { 1.0 } else { -1.0 };
                        break;
                    }
                }
            }
        }

        // Calculate intersection point
        let intersection_point = ray.pos + ray.direction * dist;

        Some(RayCollision {
            ray: ray.clone(),
            distance: dist,
            intersection: intersection_point,
            normal,
        })
    }

    /// Convert this AABB into block space, rounding down the start and rounding up the end
    pub fn to_block_aabb(&self) -> AABB<i32> {
        AABB {
            start: Point3 {
                x: self.start.x.floor() as i32,
                y: self.start.y.floor() as i32,
                z: self.start.z.floor() as i32,
            },
            end: Point3 {
                x: self.end.x.ceil() as i32,
                y: self.end.y.ceil() as i32,
                z: self.end.z.ceil() as i32,
            },
        }
    }
}

impl AABB<i32> {
    /// Iterate over the block positions covered by this AABB
    pub fn iter_blocks(&self) -> impl Iterator<Item = BlockPos> {
        (self.start.x..=self.end.x).flat_map(move |x| {
            (self.start.y..=self.end.y).flat_map(move |y| {
                (self.start.z..=self.end.z).map(move |z| BlockPos(Point3::new(x, y, z)))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Point3, Vector3};

    use crate::math::{bbox::AABB, ray::Ray};

    #[test]
    fn test_ray_aabb_intersection() {
        let aabb = AABB::new(&Point3::new(0.0, 0.0, 0.0), &Point3::new(1.0, 1.0, 1.0));

        // Ray from outside pointing into the box
        let ray1 = Ray::new(Point3::new(-1.0, 0.5, 0.5), Vector3::new(1.0, 0.0, 0.0));
        assert!(aabb.intersect_ray(&ray1).is_some());

        // Ray from outside pointing away from the box
        let ray2 = Ray::new(Point3::new(-1.0, 0.5, 0.5), Vector3::new(-1.0, 0.0, 0.0));
        assert!(aabb.intersect_ray(&ray2).is_none());

        // Ray from inside the box
        let ray3 = Ray::new(Point3::new(0.5, 0.5, 0.5), Vector3::new(1.0, 0.0, 0.0));
        assert!(aabb.intersect_ray(&ray3).is_some());
    }
}
