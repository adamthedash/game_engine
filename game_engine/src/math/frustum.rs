use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

use crate::{math::bbox::AABB, state::world::WorldPos};

/// Frustum defined by 6 planes.
/// Plane normals point inwards.
#[derive(Debug)]
pub struct Frustum {
    pub near: Plane,
    pub far: Plane,
    pub top: Plane,
    pub bottom: Plane,
    pub left: Plane,
    pub right: Plane,
}

impl Frustum {
    /// Checks if a point is within the frustum
    #[inline]
    pub fn contains_point(&self, point: &WorldPos) -> bool {
        [
            &self.near,
            &self.far,
            &self.top,
            &self.bottom,
            &self.left,
            &self.right,
        ]
        .into_iter()
        .all(|p| p.signed_distance(point) >= 0.)
    }

    /// Checks if there is any overlap between this and an AABB
    #[inline]
    pub fn intersects_aabb(&self, aabb: &AABB<f32>) -> bool {
        // https://gdbooks.gitbooks.io/3dcollisions/content/Chapter6/aabb_in_frustum.html
        // If there is any plane where the aabb is full behind it, then there's no intersection
        ![
            &self.near,
            &self.far,
            &self.top,
            &self.bottom,
            &self.left,
            &self.right,
        ]
        .into_iter()
        .any(|p| p.is_behind(aabb))
    }
}

/// Plane defined by normal & distance from origin
#[derive(Debug)]
pub struct Plane {
    // Normalised normal vector
    pub normal: Vector3<f32>,
    // Distance from origin
    pub d: f32,
}

impl Plane {
    #[inline]
    pub fn from_normal_point(normal: &Vector3<f32>, point: &Point3<f32>) -> Self {
        let normal = normal.normalize();
        let d = -normal.dot(point.to_vec());

        Self { normal, d }
    }

    /// Distance from the plane. Positive == infront
    #[inline]
    pub fn signed_distance(&self, point: &WorldPos) -> f32 {
        self.normal.dot(point.0.to_vec()) + self.d
    }

    /// Checks if an AABB is fully behind of the plane
    #[inline]
    pub fn is_behind(&self, aabb: &AABB<f32>) -> bool {
        aabb.iter_points()
            .all(|p| self.signed_distance(&WorldPos(p)) < 0.)
    }
}
