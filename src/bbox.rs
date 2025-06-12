use cgmath::{BaseNum, Point3};

#[derive(Debug)]
pub struct AABB<S: BaseNum> {
    start: Point3<S>,
    end: Point3<S>,
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
}
