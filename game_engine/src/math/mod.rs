use num_traits::Float;

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
