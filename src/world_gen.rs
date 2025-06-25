use libnoise::{Generator, ImprovedPerlin};

use crate::{
    perlin_cdf::perlin_cdf,
    world::{Biome, BlockPos, BlockType, Chunk},
};

#[derive(Debug)]
pub struct Perlin {
    // Perlin noise stuff
    source: ImprovedPerlin<3>,
    amplitudes: Vec<f64>,
    frequencies: Vec<f64>,
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
        assert!(amplitude > 0.);
        assert!(persistence > 0.);

        // Pre-generate octave values
        let mut amplitudes = vec![1.];
        let mut frequencies = vec![1. * scale];
        for i in 0..num_octaves - 1 {
            amplitudes.push(amplitudes[i] * amplitude);
            frequencies.push(frequencies[i] * persistence);
        }

        // To preserve variance, amplitudes must satisty unit circle constraint
        let divisor = amplitudes.iter().map(|a| a * a).sum::<f64>().sqrt();
        let amplitudes = amplitudes.iter().map(|a| a / divisor).collect();

        Self {
            source: libnoise::Source::improved_perlin(seed),
            amplitudes,
            frequencies,
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
            .clamp(-1., 1.)
    }
}

/// Represents an partitioned interval over -1 .. 1 that can be sampled
pub struct Intervals<T> {
    intervals: Vec<f64>,
    values: Vec<T>,
}

impl<T> Intervals<T> {
    fn new(dividers: Vec<f64>, values: Vec<T>) -> Self {
        // TODO: validate dividers
        let mut intervals = vec![-1.];
        intervals.extend(dividers);
        intervals.push(1.);

        Self { intervals, values }
    }

    fn sample(&self, t: f64) -> &T {
        assert!((-1_f64..=1.).contains(&t));

        self.intervals
            .array_windows::<2>()
            .zip(&self.values)
            .find(|([min, max], _)| (*min..=*max).contains(&t))
            .unwrap()
            .1
    }
}

pub struct ChunkGenerator {
    density: Perlin,
    biome: Perlin,
}

impl ChunkGenerator {
    pub fn new(density: Perlin, biome: Perlin) -> Self {
        Self { density, biome }
    }

    pub fn generate_chunk(&self, world_pos: BlockPos) -> Chunk {
        let biome_interval = Intervals::new(
            [0.33, 0.66].into_iter().map(perlin_cdf).collect(),
            vec![Biome::DirtLand, Biome::StoneLand, Biome::DenseCaves],
        );

        let dirtland_interval = Intervals::new(
            [0.05, 0.5, 0.55, 0.65]
                .into_iter()
                .map(perlin_cdf)
                .collect(),
            vec![
                BlockType::VoidStone,
                BlockType::Air,
                BlockType::Dirt,
                BlockType::MossyStone,
                BlockType::Stone,
            ],
        );
        let stone_interval = Intervals::new(
            [0.05, 0.5, 0.75].into_iter().map(perlin_cdf).collect(),
            vec![
                BlockType::RadioactiveStone,
                BlockType::Air,
                BlockType::Stone,
                BlockType::DarkStone,
            ],
        );
        let dense_caves_interval = Intervals::new(
            [0.2, 0.3].into_iter().map(perlin_cdf).collect(),
            vec![BlockType::Air, BlockType::Stone, BlockType::DarkStone],
        );

        // TODO: Perf - uninit array
        let mut blocks =
            [[[BlockType::Air; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE];
        for (i, x) in (world_pos.0.x..).take(Chunk::CHUNK_SIZE).enumerate() {
            for (j, y) in (world_pos.0.y..).take(Chunk::CHUNK_SIZE).enumerate() {
                for (k, z) in (world_pos.0.z..).take(Chunk::CHUNK_SIZE).enumerate() {
                    let density = self.density.sample(x as f64, y as f64, z as f64);
                    let biome = self.biome.sample(x as f64, y as f64, z as f64);

                    let biome_type = biome_interval.sample(biome);
                    let sampler = match biome_type {
                        Biome::DirtLand => &dirtland_interval,
                        Biome::StoneLand => &stone_interval,
                        Biome::DenseCaves => &dense_caves_interval,
                    };

                    blocks[i][j][k] = *sampler.sample(density);
                }
            }
        }

        let (chunk_pos, _) = world_pos.to_chunk_offset();
        Chunk {
            chunk_pos,
            world_pos,
            blocks,
            // WARN: exposed_blocks must be populated elsewhere as chunk-to-chunk info is needed
            exposed_blocks: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {

    use rand::random_range;

    use super::Perlin;

    #[test]
    fn test_perlin() {
        let generator = Perlin::new(42, 3, 2., 2., 1. / 16.);
        println!("generator: {generator:?}");
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
    fn test_perlin_dist() {
        let generator = Perlin::new(42, 5, 0.5, 2., 1. / 16.);

        let n = 1000000;
        let mut vals = (0..n)
            .map(|_| {
                let x = random_range(-1e10..1e10);
                let y = random_range(-1e10..1e10);
                let z = random_range(-1e10..1e10);

                generator.sample(x, y, z)
            })
            .collect::<Vec<_>>();
        vals.sort_unstable_by(|a, b| a.total_cmp(b));

        let mut percentiles = [1e-5, 1e-4, 1e-3, 1e-2]
            .into_iter()
            // Extremes
            .flat_map(|x| [x, 1. - x])
            // 0-100 to 1 dp
            .chain((0..=1000).map(|i| i as f64 / 1000.))
            .collect::<Vec<_>>();
        percentiles.sort_unstable_by(|a, b| a.total_cmp(b));

        let cdf_values = percentiles
            .iter()
            .map(|p| {
                let index = ((n - 1) as f64 * p) as usize;
                vals[index]
            })
            .collect::<Vec<_>>();

        // Save out to file
        percentiles
            .iter()
            .zip(cdf_values)
            .for_each(|(p, v)| println!("({p},{v}),"));
    }
}
