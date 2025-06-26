use libnoise::{Generator, ImprovedPerlin};

use crate::world::{BlockPos, Chunk};

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
#[derive(Clone)]
pub struct Intervals<T: Clone> {
    intervals: Vec<f64>,
    values: Vec<T>,
}

impl<T: Clone> Intervals<T> {
    pub fn new(dividers: Vec<f64>, values: Vec<T>) -> Self {
        assert!(dividers.is_sorted());
        assert!(dividers.len() == values.len() - 1);
        assert!(dividers.iter().all(|d| { (-1_f64..=1.).contains(d) }));

        let mut intervals = vec![-1.];
        intervals.extend(dividers);
        intervals.push(1.);

        Self { intervals, values }
    }

    pub fn sample(&self, t: f64) -> &T {
        assert!((-1_f64..=1.).contains(&t));

        self.intervals
            .array_windows::<2>()
            .zip(&self.values)
            .find(|([min, max], _)| (*min..=*max).contains(&t))
            .unwrap()
            .1
    }
}

pub trait ChunkGenerator {
    fn generate_chunk(&self, world_pos: BlockPos) -> Chunk;
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
