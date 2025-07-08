use cgmath::Point3;

pub fn hash_pos(world_seed: u32, pos: Point3<i32>) -> u32 {
    world_seed
        .wrapping_mul(0x9e3779b9)
        .wrapping_add(pos.x as u32)
        .wrapping_mul(0x9e3779b9)
        .wrapping_add(pos.y as u32)
        .wrapping_mul(0x9e3779b9)
        .wrapping_add(pos.z as u32)
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
