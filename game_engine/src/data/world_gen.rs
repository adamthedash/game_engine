use enum_map::{EnumMap, enum_map};

use crate::{
    data::{biome::Biome, block::BlockType},
    perlin_cdf::perlin_cdf,
    world::{BlockPos, Chunk},
    world_gen::{ChunkGenerator, Intervals, Perlin},
};

pub struct DefaultGenerator {
    density: Perlin,
    biome: Perlin,
    biome_selector: Intervals<Biome>,
    block_selectors: EnumMap<Biome, Intervals<BlockType>>,
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

        Self {
            density,
            biome,
            biome_selector,
            block_selectors,
        }
    }
}

impl ChunkGenerator for DefaultGenerator {
    fn generate_chunk(&self, world_pos: BlockPos) -> Chunk {
        // TODO: Perf - uninit array
        let mut blocks =
            [[[BlockType::Air; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE];

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
        Chunk {
            chunk_pos,
            world_pos,
            blocks,
            // WARN: exposed_blocks must be populated elsewhere as chunk-to-chunk info is needed
            exposed_blocks: Default::default(),
        }
    }
}
