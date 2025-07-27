use cgmath::{InnerSpace, Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    #[inline]
    pub fn new(pos: Point3<f32>, direction: Vector3<f32>) -> Self {
        assert!(direction.magnitude2() > 0.);
        Self {
            pos,
            direction: direction.normalize(),
        }
    }

    /// Return the point a given distance along the ray
    #[inline]
    pub fn project(&self, distance: f32) -> Point3<f32> {
        self.pos + self.direction * distance
    }
}

#[derive(Debug)]
pub struct RayCollision {
    pub ray: Ray,
    pub distance: f32,
    pub intersection: Point3<f32>,
    pub normal: Vector3<f32>,
}
