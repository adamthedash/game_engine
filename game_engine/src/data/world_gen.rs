use enum_map::{EnumMap, enum_map};
use num_traits::Euclid;

use crate::{
    data::{biome::Biome, block::BlockType},
    math::{LCG, hash_pos},
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
        let world_seed = 123;
        let seed_points_per_chunk = 10;
        let seed = hash_pos(world_seed, chunk_pos.0);
        let seed_points = LCG::new(seed)
            .map(|val| {
                // Convert to random float 0-1
                let rand = val as f32 / (1 << 30) as f32;

                // Convert to position
                let val = val as usize % Chunk::BLOCKS_PER_CHUNK;
                let (rem, x) = val.div_rem_euclid(&Chunk::CHUNK_SIZE);
                let (y, z) = rem.div_rem_euclid(&Chunk::CHUNK_SIZE);

                (rand, (x, y, z))
            })
            .take(seed_points_per_chunk)
            // Only generate ores in the ground
            .filter(|(_, (x, y, z))| blocks[*x][*y][*z] != BlockType::Air)
            .collect::<Vec<_>>();

        seed_points.into_iter().for_each(|(rand, (x, y, z))| {
            blocks[x][y][z] = *self.ore_selector.sample(rand as f64);
        });

        Chunk {
            chunk_pos,
            world_pos,
            blocks,
            // WARN: exposed_blocks must be populated elsewhere as chunk-to-chunk info is needed
            exposed_blocks: Default::default(),
        }
    }
}
