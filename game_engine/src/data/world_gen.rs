use cgmath::Vector3;
use enum_map::{EnumMap, enum_map};
use itertools::izip;
use num_traits::Euclid;

use crate::{
    data::{biome::Biome, block::BlockType},
    math::rng::{LCG, hash_pos, split_seed_arr, split_seed_iter},
    perlin_cdf::perlin_cdf,
    state::world::{BlockPos, Chunk},
    world_gen::{ChunkGenerator, Intervals, Perlin},
};

pub struct DefaultGenerator {
    density: Perlin,
    biome: Perlin,
    biome_selector: Intervals<Biome>,
    block_selectors: EnumMap<Biome, Intervals<BlockType>>,
    ore_selector: Intervals<BlockType>,
}
impl DefaultGenerator {
    pub fn new(density: Perlin, biome: Perlin) -> Self {
        let biome_selector = Intervals::new(
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
        let block_selectors = enum_map! {
            Biome::DirtLand => dirtland_interval.clone(),
            Biome::StoneLand => stone_interval.clone(),
            Biome::DenseCaves => dense_caves_interval.clone(),
        };

        let ore_selector = Intervals::new(
            vec![0.25, 0.5, 0.8, 0.95],
            vec![
                BlockType::Copper,
                BlockType::Tin,
                BlockType::Coal,
                BlockType::Iron,
                BlockType::MagicMetal,
            ],
        );

        Self {
            density,
            biome,
            biome_selector,
            block_selectors,
            ore_selector,
        }
    }
}

impl ChunkGenerator for DefaultGenerator {
    fn generate_chunk(&self, world_pos: BlockPos) -> Chunk {
        // TODO: Perf - uninit array
        let mut blocks =
            [[[BlockType::Air; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE];

        // Base terrain
        for (i, x) in (world_pos.0.x..).take(Chunk::CHUNK_SIZE).enumerate() {
            for (j, y) in (world_pos.0.y..).take(Chunk::CHUNK_SIZE).enumerate() {
                for (k, z) in (world_pos.0.z..).take(Chunk::CHUNK_SIZE).enumerate() {
                    let density = self.density.sample(x as f64, y as f64, z as f64);
                    let biome = self.biome.sample(x as f64, y as f64, z as f64);

                    let biome_type = self.biome_selector.sample(biome);
                    let block_type = self.block_selectors[*biome_type].sample(density);

                    blocks[i][j][k] = *block_type;
                }
            }
        }

        let (chunk_pos, _) = world_pos.to_chunk_offset();

        // Ore generation
        // Generate seed points
        // TODO: lift these parameters out
        let world_seed = 123;
        let seed_points_per_chunk = 10;
        let blocks_per_vein = 10;

        let chunk_seed = hash_pos(world_seed, chunk_pos.0);
        let [point_seed, rarity_seed, vein_seed] = split_seed_arr(chunk_seed);
        let seed_points = LCG::new(point_seed)
            .map(|val| {
                // Convert to position
                let val = val as usize % Chunk::BLOCKS_PER_CHUNK;
                let (rem, x) = val.div_rem_euclid(&Chunk::CHUNK_SIZE);
                let (y, z) = rem.div_rem_euclid(&Chunk::CHUNK_SIZE);

                (x, y, z)
            })
            .take(seed_points_per_chunk)
            // Only generate ores in the ground
            .filter(|(x, y, z)| blocks[*x][*y][*z] != BlockType::Air)
            .collect::<Vec<_>>();

        let rarity_values = LCG::new(rarity_seed).map(|val| {
            // Convert to random float 0-1
            val as f32 / (1 << 30) as f32
        });

        izip!(seed_points, rarity_values, split_seed_iter(vein_seed)).for_each(
            |((x, y, z), rarity, vein_seed)| {
                let block_type = *self.ore_selector.sample(rarity as f64);

                // Set the seed point
                blocks[x][y][z] = block_type;
                let mut points_generated = vec![(x, y, z)];

                // Init RNGs for this vein
                let [offset_seed, selection_seed] = split_seed_arr(vein_seed);
                let mut offset_generator = LCG::new(offset_seed).map(|val| {
                    Chunk::ADJACENT_OFFSETS[val as usize % Chunk::ADJACENT_OFFSETS.len()]
                });
                let mut selection_generator = LCG::new(selection_seed);

                // Attempt to grow the vein
                for _ in 0..blocks_per_vein {
                    // Select an existing block
                    let (x, y, z) = points_generated
                        [selection_generator.next().unwrap() as usize % points_generated.len()];
                    let block_pos = &world_pos + Vector3::new(x as i32, y as i32, z as i32);

                    // Select a block next to it
                    let offset = offset_generator.next().unwrap();
                    let adjacent_pos = block_pos + offset;

                    // If we've gone over chunk boundaries, discard
                    // TODO: Once cross-boundary generation is implemented, get rid of this check
                    let (adjacent_chunk, adjacent_offset) = adjacent_pos.to_chunk_offset();
                    if adjacent_chunk != chunk_pos {
                        continue;
                    }

                    // If we've gone back on ourselves, discard it
                    if points_generated.contains(&adjacent_offset) {
                        continue;
                    }

                    // Don't generate the ores in air
                    let (x, y, z) = adjacent_offset;
                    if blocks[x][y][z] == BlockType::Air {
                        continue;
                    }

                    // Grow the vein
                    blocks[x][y][z] = block_type;

                    points_generated.push(adjacent_offset);
                }
            },
        );

        Chunk {
            chunk_pos,
            world_pos,
            blocks,
            // WARN: exposed_blocks must be populated elsewhere as chunk-to-chunk info is needed
            exposed_blocks: Default::default(),
        }
    }
}
