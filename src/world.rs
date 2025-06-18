use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use cgmath::{InnerSpace, Point3, Vector3};
use glob::glob;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{Euclid, FromPrimitive, ToPrimitive};
use rustc_hash::FxHashMap;

use crate::{
    bbox::AABB,
    block::Block,
    world_gen::{ChunkGenerator, Perlin},
};

/// Represents the position of a chunk in chunk-space (1 unit moves 1 chunk length)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ChunkPos(pub Point3<i32>);

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(Point3::new(x, y, z))
    }

    /// Convert to the position of the chunk's origin block
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos(self.0 * Chunk::CHUNK_SIZE as i32)
    }

    /// Iterates over all chunk positions within a given circular distance
    pub fn chunks_within(&self, num_chunks: u32) -> impl Iterator<Item = ChunkPos> {
        let num_chunks = num_chunks as i32;
        let dist2 = num_chunks * num_chunks;

        let offsets = (-num_chunks..=num_chunks)
            .flat_map(move |x| {
                (-num_chunks..=num_chunks).flat_map(move |y| {
                    (-num_chunks..=num_chunks).map(move |z| Vector3::new(x, y, z))
                })
            })
            .filter(move |offset| offset.magnitude2() <= dist2);
        offsets.map(move |offset| ChunkPos(self.0 + offset))
    }

    pub fn aabb(&self) -> AABB<i32> {
        AABB::new(
            &self.to_block_pos().0,
            &Self(self.0 + Vector3::new(1, 1, 1)).to_block_pos().0,
        )
    }
}

/// Represents the position of a block in block-space (1 unit moves 1 block length)
#[derive(Debug, Clone)]
pub struct BlockPos(pub Point3<i32>);

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(Point3::new(x, y, z))
    }

    /// Convert to a chunk position and the block position relative to the chunk
    pub fn to_chunk_offset(&self) -> (ChunkPos, (i32, i32, i32)) {
        let chunk_index = ChunkPos::new(
            self.0.x.div_euclid(Chunk::CHUNK_SIZE as i32),
            self.0.y.div_euclid(Chunk::CHUNK_SIZE as i32),
            self.0.z.div_euclid(Chunk::CHUNK_SIZE as i32),
        );
        let within_chunk_pos = (
            self.0.x.rem_euclid(Chunk::CHUNK_SIZE as i32),
            self.0.y.rem_euclid(Chunk::CHUNK_SIZE as i32),
            self.0.z.rem_euclid(Chunk::CHUNK_SIZE as i32),
        );

        (chunk_index, within_chunk_pos)
    }

    pub fn to_world_pos(&self) -> WorldPos {
        WorldPos(self.0.cast().expect("Failed to cast BlockPos -> WorldPos"))
    }

    /// Centre point of the block in world space
    pub fn centre(&self) -> WorldPos {
        WorldPos(self.to_world_pos().0 + Vector3::new(0.5, 0.5, 0.5))
    }
}

/// Represents any position in the world in block-space (1 unit moves 1 block length)
#[derive(Debug, Copy, Clone)]
pub struct WorldPos(pub Point3<f32>);

impl WorldPos {
    /// Convert to a block position, rounding down
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(
            self.0.x.floor() as i32,
            self.0.y.floor() as i32,
            self.0.z.floor() as i32,
        )
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air = 0,
    Dirt,
    Stone,
    Smiley,
    Smiley2,
}

#[derive(Debug)]
pub struct Chunk {
    pub chunk_pos: ChunkPos, // Position of chunk in chunk space
    pub world_pos: BlockPos, // Position of corner block in world space
    pub blocks: [[[BlockType; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE], // Block type IDs
    pub exposed_blocks: [[[bool; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE]; Chunk::CHUNK_SIZE],
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 16;
    pub const BLOCKS_PER_CHUNK: usize = const { Self::CHUNK_SIZE.pow(3) };

    /// Relative offsets in cardinal directions
    pub const ADJACENT_OFFSETS: [Vector3<i32>; 6] = [
        Vector3::new(-1, 0, 0), // left
        Vector3::new(1, 0, 0),  // right
        Vector3::new(0, -1, 0), // down
        Vector3::new(0, 1, 0),  // up
        Vector3::new(0, 0, -1), // backwards
        Vector3::new(0, 0, 1),  // forwards
    ];

    /// Cube corners
    pub const CORNER_OFFSETS: [Vector3<i32>; 8] = [
        Vector3::new(0, 0, 0),
        Vector3::new(0, 0, 1),
        Vector3::new(0, 1, 0),
        Vector3::new(0, 1, 1),
        Vector3::new(1, 0, 0),
        Vector3::new(1, 0, 1),
        Vector3::new(1, 1, 0),
        Vector3::new(1, 1, 1),
    ];

    /// Iterate over the blocks in this chunk
    pub fn iter_blocks(&self) -> ChunkIter<'_> {
        ChunkIter {
            chunk: self,
            index: 0,
        }
    }

    /// Get a reference to a block in this chunk
    pub fn get_block(&self, pos: (i32, i32, i32)) -> &BlockType {
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.0));
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.1));
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.2));

        &self.blocks[pos.0 as usize][pos.1 as usize][pos.2 as usize]
    }

    /// Get a reference to a block in this chunk
    pub fn get_block_mut(&mut self, pos: (i32, i32, i32)) -> &mut BlockType {
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.0));
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.1));
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.2));

        &mut self.blocks[pos.0 as usize][pos.1 as usize][pos.2 as usize]
    }

    pub fn is_block_exposed(&self, pos: (i32, i32, i32)) -> bool {
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.0));
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.1));
        assert!((0..Self::CHUNK_SIZE as i32).contains(&pos.2));

        self.exposed_blocks[pos.0 as usize][pos.1 as usize][pos.2 as usize]
    }
}

pub struct ChunkIter<'a> {
    chunk: &'a Chunk,
    index: usize,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= Chunk::BLOCKS_PER_CHUNK {
            return None;
        }

        // Convert the iterator index into the corresponding block position
        let (rem, x) = self.index.div_rem_euclid(&Chunk::CHUNK_SIZE);
        let (z, y) = rem.div_rem_euclid(&Chunk::CHUNK_SIZE);
        let block_pos =
            BlockPos(self.chunk.world_pos.0 + Vector3::new(x as i32, y as i32, z as i32));

        self.index += 1;

        Some(Block {
            block_pos,
            block_type: self.chunk.blocks[x][y][z],
        })
    }
}

/// All of the world data
pub struct World {
    // Generated chunks
    pub chunks: FxHashMap<ChunkPos, Chunk>,
    pub generator: ChunkGenerator,
}

impl World {
    /// Save the world data to disk
    /// 1 chunk = 1 file, block types stored as a flat array
    pub fn save(&self, folder: &Path) {
        if folder.exists() {
            fs::remove_dir_all(folder).expect("Failed to remove save folder");
        }
        assert!(!folder.exists());
        fs::create_dir(folder).expect("Failed to create save folder");
        self.chunks.iter().for_each(|(pos, chunk)| {
            let serialised = chunk
                .blocks
                .iter()
                .flatten()
                .flatten()
                .flat_map(|x| x.to_u16().unwrap().to_le_bytes())
                .collect::<Vec<_>>();

            let filename = folder.join(format!("{}_{}_{}.chunk", pos.0.x, pos.0.y, pos.0.z));
            let mut writer =
                BufWriter::new(File::create_new(&filename).expect("Failed to create file"));
            writer
                .write_all(&serialised)
                .expect("Failed to save chunk file");
        });
    }

    pub fn load(folder: &Path) -> Self {
        assert!(folder.is_dir());
        let chunks = glob(&format!("{}/*.chunk", folder.to_str().unwrap()))
            .unwrap()
            .map(|f| {
                let filename = f.unwrap();
                let [x, y, z] = filename
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split("_")
                    .map(|s| s.parse::<i32>().unwrap())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                let chunk_pos = ChunkPos::new(x, y, z);

                let blocks = fs::read(&filename)
                    .unwrap()
                    .chunks_exact(std::mem::size_of::<u16>())
                    .map(|c| {
                        BlockType::from_u16(u16::from_le_bytes(c.try_into().unwrap())).unwrap()
                    })
                    .collect::<Vec<_>>()
                    .chunks_exact(Chunk::CHUNK_SIZE)
                    .map(|c| c.try_into().unwrap())
                    .collect::<Vec<[_; Chunk::CHUNK_SIZE]>>()
                    .chunks_exact(Chunk::CHUNK_SIZE)
                    .map(|c| c.try_into().unwrap())
                    .collect::<Vec<[_; Chunk::CHUNK_SIZE]>>()
                    .try_into()
                    .unwrap();

                Chunk {
                    world_pos: chunk_pos.to_block_pos(),
                    chunk_pos,
                    blocks,
                    exposed_blocks: Default::default(),
                }
            })
            .fold(FxHashMap::default(), |mut hm, chunk| {
                hm.insert(chunk.chunk_pos.clone(), chunk);
                hm
            });

        let chunk_gen = ChunkGenerator::new(Perlin::new(42, 3, 0.5, 2., 1. / 64.));

        let mut world = World {
            chunks,
            generator: chunk_gen,
        };
        world.update_all_exposed_blocks();
        world
    }

    pub fn default() -> Self {
        // This is a nice one
        let chunk_gen = ChunkGenerator::new(Perlin::new(42, 4, 1., 0.5, 1. / 16.));

        Self {
            chunks: Default::default(),
            generator: chunk_gen,
        }
    }

    fn update_all_exposed_blocks(&mut self) {
        let chunks_to_update = self.chunks.keys().cloned().collect::<Vec<_>>();
        chunks_to_update
            .iter()
            .for_each(|pos| self.update_exposed_blocks(pos));
    }

    /// Propogate exposure information to the cache
    pub fn update_exposed_blocks(&mut self, chunk_pos: &ChunkPos) {
        let blocks_to_update = self
            .chunks
            .get(chunk_pos)
            .expect("Chunk doesn't exist!")
            .iter_blocks()
            .filter(|b| b.block_type == BlockType::Air)
            .collect::<Vec<_>>();

        blocks_to_update.iter().for_each(|b| {
            Chunk::ADJACENT_OFFSETS.iter().for_each(|o| {
                let (chunk_pos, (x, y, z)) = BlockPos(b.block_pos.0 + o).to_chunk_offset();

                if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                    chunk.exposed_blocks[x as usize][y as usize][z as usize] = true;
                }
            });
        });
    }

    /// Check if the given block has any side that isn't surrounded
    pub fn is_block_exposed(&self, pos: &BlockPos) -> bool {
        let (chunk_pos, (x, y, z)) = pos.to_chunk_offset();
        self.chunks
            .get(&chunk_pos)
            .expect("Chunk doesn't exist!")
            .exposed_blocks[x as usize][y as usize][z as usize]
    }

    /// Returns a reference to a chunk, or generates if it doesn't exist
    pub fn get_or_generate_chunk(&mut self, pos: &ChunkPos) -> &Chunk {
        if !self.chunks.contains_key(pos) {
            // Create a new chunk
            let new_chunk = self.generator.generate_chunk(pos.to_block_pos());
            self.chunks.insert(pos.clone(), new_chunk);

            // Add exposed block cache
            self.update_exposed_blocks(pos);

            // Also need to re-run adjacent exposure tests
            let chunks_to_update = Chunk::ADJACENT_OFFSETS
                .iter()
                .map(|o| ChunkPos(pos.0 + o))
                .filter(|pos| self.chunks.contains_key(pos))
                .collect::<Vec<_>>();
            chunks_to_update.iter().for_each(|pos| {
                self.update_exposed_blocks(pos);
            });
        }
        self.chunks.get(pos).expect("Chunk not found!")
    }

    pub fn get_block(&self, pos: &BlockPos) -> Option<Block> {
        let (chunk_pos, offset) = pos.to_chunk_offset();
        self.chunks.get(&chunk_pos).map(|chunk| Block {
            block_pos: pos.clone(),
            block_type: *chunk.get_block(offset),
        })
    }

    pub fn get_block_mut(&mut self, pos: &BlockPos) -> Option<&mut BlockType> {
        let (chunk_pos, offset) = pos.to_chunk_offset();
        self.chunks
            .get_mut(&chunk_pos)
            .map(|chunk| chunk.get_block_mut(offset))
    }
}
