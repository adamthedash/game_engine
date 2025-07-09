use cgmath::{BaseNum, InnerSpace, Point3};

use crate::render::ray::Ray;

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
}

impl AABB<f32> {
    /// Check if a ray intersects with this AABB. Returns the distance to the intersection point
    /// if it does.
    /// https://en.wikipedia.org/wiki/Slab_method
    pub fn intersect_ray(&self, ray: &Ray) -> Option<f32> {
        assert!(ray.direction.magnitude2() > 0.);
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        // Iterate over all 3 axes
        for i in 0..3 {
            if ray.direction[i].abs() < f32::EPSILON {
                // Ray is parallel to the slab
                if ray.pos[i] < self.start[i] || ray.pos[i] > self.end[i] {
                    return None; // Ray is outside the slab and parallel to it
                }
            }

            let inv_dir = 1.0 / ray.direction[i];
            let mut t1 = (self.start[i] - ray.pos[i]) * inv_dir;
            let mut t2 = (self.end[i] - ray.pos[i]) * inv_dir;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            tmin = tmin.max(t1);
            tmax = tmax.min(t2);

            if tmin > tmax {
                return None;
            }
        }

        // Return the closest intersection point that's in front of the ray
        if tmax >= 0.0 {
            Some(if tmin >= 0.0 { tmin } else { tmax })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Point3, Vector3};

    use crate::{bbox::AABB, render::ray::Ray};

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
