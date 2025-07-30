use cgmath::{Angle, BaseFloat, InnerSpace, Rad, Vector3, Zero};
use num_traits::{Float, float::TotalOrder};

pub mod bbox;
pub mod frustum;
pub mod ray;
pub mod rng;

pub trait NumFuncs: Float {
    /// Returns the futher of the two numbers from 0
    #[inline]
    fn further(&self, other: Self) -> Self {
        assert_eq!(
            self.is_sign_negative(),
            other.is_sign_negative(),
            "Numbers must be on the same side of 0"
        );

        if self.is_sign_negative() {
            self.min(other)
        } else {
            self.max(other)
        }
    }

    /// Returns the futher of the two numbers from 0
    #[inline]
    fn closer(&self, other: Self) -> Self {
        assert_eq!(
            self.is_sign_negative(),
            other.is_sign_negative(),
            "Numbers must be on the same side of 0"
        );

        if self.is_sign_negative() {
            self.max(other)
        } else {
            self.min(other)
        }
    }

    /// Shrink towards 0 by the given amount, clipping at 0
    #[inline]
    fn shrink(&self, amount: Self) -> Self {
        if self.is_sign_negative() {
            (*self + amount).min(Self::neg_zero())
        } else {
            (*self - amount).max(Self::zero())
        }
    }

    /// Expand away from 0 by the given amount, clipping at 0
    #[inline]
    fn expand(&self, amount: Self) -> Self {
        self.shrink(-amount)
    }
}

impl<T: Float> NumFuncs for T {}

pub trait Vectorfuncs {
    /// Convert a vector offset to it's closest unit offset along one cardinal direction
    fn to_cardinal_offset(&self) -> Vector3<i32>;
}

impl<S: BaseFloat + TotalOrder> Vectorfuncs for Vector3<S> {
    #[inline]
    fn to_cardinal_offset(&self) -> Vector3<i32> {
        // Get the axis with the largest magnitude
        let largest_mag = [self[0], self[1], self[2]]
            .into_iter()
            .enumerate()
            .max_by(|(_, x1), (_, x2)| x1.abs().total_cmp(&x2.abs()))
            .map(|(i, _)| i)
            .unwrap();

        // Get the unit vector along this axis
        let mut offset = Vector3::zero();
        offset[largest_mag] = if self[largest_mag].is_sign_positive() {
            1
        } else {
            -1
        };

        offset
    }
}

#[inline]
pub fn angles_to_vec3(yaw: Rad<f32>, pitch: Rad<f32>) -> Vector3<f32> {
    let (y, verticality) = pitch.sin_cos();
    let (z, x) = yaw.sin_cos();
    Vector3::new(x * verticality, y, z * verticality).normalize()
}
