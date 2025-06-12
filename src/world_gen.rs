use libnoise::{Generator, ImprovedPerlin};

use crate::world::{BlockPos, BlockType, Chunk};

#[derive(Debug)]
pub struct Perlin {
    // Perlin noise stuff
    source: ImprovedPerlin<3>,
    amplitudes: Vec<f64>,
    frequencies: Vec<f64>,
    divisor: f64,
}

impl Perlin {
    ///
    /// num_octaves: Number of noise layers to use
    /// amplitude: Influence multiplier for each subsequent noise layer
    /// persistence: Granularity multiplier for each subsequent noise layer
    /// scale: Overall granularity multiplier, Bigger == more granular
    ///
    pub fn new(
        seed: u64,
        num_octaves: usize,
        amplitude: f64,
        persistence: f64,
        scale: f64,
    ) -> Self {
        assert!(num_octaves > 0);

        // Pre-generate octave values
        let mut amplitudes = vec![1.];
        let mut frequencies = vec![1. * scale];
        for i in 0..num_octaves - 1 {
            amplitudes.push(amplitudes[i] * amplitude);
            frequencies.push(frequencies[i] * persistence);
        }
        let divisor = amplitudes.iter().sum();

        Self {
            source: libnoise::Source::improved_perlin(seed),
            amplitudes,
            frequencies,
            divisor,
        }
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        self.frequencies
            .iter()
            .zip(&self.amplitudes)
            .map(|(f, a)| {
                self.source.sample([
                    // Need to mod here to handle negatives as the libnoise crate doesn't do this internally
                    (x * f).rem_euclid(256.),
                    (y * f).rem_euclid(256.),
                    (z * f).rem_euclid(256.),
                ]) * a
            })
            .sum::<f64>()
            / self.divisor
    }
}

pub struct ChunkGenerator {
    rng: Perlin,
}

impl ChunkGenerator {
    pub fn new(rng: Perlin) -> Self {
        Self { rng }
    }

    pub fn generate_chunk(&self, world_pos: BlockPos) -> Chunk {
        // TODO: Perf - uninit array
        let mut blocks =
            [[[BlockType::Air; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE];
        for (i, x) in (world_pos.0..).take(Chunk::CHUNK_SIZE).enumerate() {
            for (j, y) in (world_pos.1..).take(Chunk::CHUNK_SIZE).enumerate() {
                for (k, z) in (world_pos.2..).take(Chunk::CHUNK_SIZE).enumerate() {
                    let density = self.rng.sample(x as f64, y as f64, z as f64);

                    // Treat random number as if it was density
                    let block_type = match density {
                        -1_f64..0. => BlockType::Air,
                        0_f64..0.25 => BlockType::Dirt,
                        0.25_f64..1. => BlockType::Stone,
                        _ => unreachable!("Random number generated outside of -1 .. 1 !"),
                    };

                    blocks[i][j][k] = block_type;
                }
            }
        }

        let (chunk_pos, _) = world_pos.to_chunk_offset();
        Chunk {
            chunk_pos,
            world_pos,
            blocks,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{ChunkGenerator, Perlin};
    use crate::world::BlockPos;

    #[test]
    fn test_perlin() {
        let generator = Perlin::new(42, 3, 2., 2., 1. / 16.);
        println!("generator: {:?}", generator);
        for x in 0..64 {
            for z in 0..64 {
                let val = generator.sample(x as f64, 0., z as f64);
                let c = match val {
                    -1_f64..0. => ".",
                    0_f64..=1. => "#",
                    _ => " ",
                };
                print!("{c}");
            }
            println!();
        }
    }

    #[test]
    fn test_chunk() {
        let generator = Perlin::new(42, 3, 2., 2., 1. / 16.);
        let chunk_gen = ChunkGenerator::new(generator);

        let chunk = chunk_gen.generate_chunk(BlockPos(0, 0, 0));
        println!("{:?}", chunk);
    }

    #[test]
    fn test_perlin_image() {
        let generator = Perlin::new(42, 3, 0.5, 2., 1. / 64.);

        let xs = -128..128;
        let zs = -128..128;
        //let xs = 0..256;
        //let zs = 0..256;

        let mut img = image::GrayImage::new(xs.len() as u32, zs.len() as u32);

        for (i, x) in xs.enumerate() {
            for (j, z) in zs.clone().enumerate() {
                let val = generator.sample(x as f64, 0., z as f64);
                let val = ((val + 1.) * 128.).clamp(0., 255.) as u8;

                img.put_pixel(i as u32, j as u32, image::Luma([val]));
            }
        }

        img.save("map.png").unwrap();
    }
}
