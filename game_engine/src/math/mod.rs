pub mod bbox;
pub mod frustum;
pub mod ray;
pub mod rng;

/// Returns the futher of the two numbers from 0
#[inline]
pub fn further(a: f32, b: f32) -> f32 {
    assert_eq!(a.signum(), b.signum());

    if a < 0. { a.min(b) } else { a.max(b) }
}

/// Returns the closer of the two numbers from 0
#[inline]
pub fn closer(a: f32, b: f32) -> f32 {
    assert_eq!(a.signum(), b.signum());

    if a < 0. { a.max(b) } else { a.min(b) }
}
