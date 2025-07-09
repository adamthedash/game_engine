use cgmath::Point3;
use itertools::Itertools;

/// Create a unique seed for a point in the world
pub fn hash_pos(world_seed: u32, pos: Point3<i32>) -> u32 {
    world_seed
        .wrapping_mul(0x9e3779b9)
        .wrapping_add(pos.x as u32)
        .wrapping_mul(0x9e3779b9)
        .wrapping_add(pos.y as u32)
        .wrapping_mul(0x9e3779b9)
        .wrapping_add(pos.z as u32)
}

/// Split a random seed into multiple sub-seeds
pub fn split_seed_arr<const N: usize>(seed: u32) -> [u32; N] {
    Splitmix64::new(seed as u64)
        .take(N)
        .map(|x| x as u32)
        .collect_array()
        .unwrap()
}

/// Split a random seed into multiple sub-seeds
pub fn split_seed_iter(seed: u32) -> impl Iterator<Item = u32> {
    Splitmix64::new(seed as u64).map(|x| x as u32)
}

/// RNG used for splitting seeds
/// https://rosettacode.org/wiki/Pseudo-random_numbers/Splitmix64
struct Splitmix64 {
    state: u64,
}

impl Splitmix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl Iterator for Splitmix64 {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        self.state = (self.state ^ (self.state >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        self.state = (self.state ^ (self.state >> 27)).wrapping_mul(0x94d049bb133111eb);
        self.state ^= self.state >> 31;

        Some(self.state)
    }
}

/// Linear Congruential Generator
/// https://en.wikipedia.org/wiki/Linear_congruential_generator
pub struct LCG {
    state: u32,
    a: u32,
    c: u32,
    m: u32,
}

impl LCG {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            // Borland
            a: 22695477,
            c: 1,
            m: 1 << 30,
        }
    }
}

impl Iterator for LCG {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.state = (self.state * self.a + self.c) % self.m;
        Some(self.state)
    }
}
