use cgmath::{InnerSpace, Point3, Vector3};

#[derive(Debug)]
pub struct Ray {
    pub pos: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    pub fn new(pos: Point3<f32>, direction: Vector3<f32>) -> Self {
        assert!(direction.magnitude2() > 0.);
        Self {
            pos,
            direction: direction.normalize(),
        }
    }
}
