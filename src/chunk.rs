use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use glob::glob;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{Euclid, FromPrimitive, ToPrimitive};
use rustc_hash::FxHashMap;

use crate::{
    block::Block,
    world_gen::{ChunkGenerator, Perlin},
};

// Represents the position of a chunk in chunk-space (1 unit moves 1 chunk length)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ChunkPos(pub i32, pub i32, pub i32);

impl ChunkPos {
    pub fn to_world_pos(&self) -> WorldPos {
        WorldPos(
            self.0 * Chunk::CHUNK_SIZE as i32,
            self.1 * Chunk::CHUNK_SIZE as i32,
            self.2 * Chunk::CHUNK_SIZE as i32,
        )
    }

    /// Iterates over all chunk positions within a given circular distance
    pub fn chunks_within(&self, num_chunks: u32) -> impl Iterator<Item = ChunkPos> {
        let num_chunks = num_chunks as i32;
        let dist2 = num_chunks * num_chunks;

        let offsets = (-num_chunks..=num_chunks)
            .flat_map(move |x| {
                (-num_chunks..=num_chunks)
                    .flat_map(move |y| (-num_chunks..=num_chunks).map(move |z| (x, y, z)))
            })
            .filter(move |(x, y, z)| x * x + y * y + z * z <= dist2);
        offsets.map(move |(x, y, z)| ChunkPos(self.0 + x, self.1 + y, self.2 + z))
    }
}

// Represents the position of a block in world-space (1 unit moves 1 block length)
#[derive(Debug)]
pub struct WorldPos(pub i32, pub i32, pub i32);

impl WorldPos {
    pub fn to_chunk_offset(&self) -> (ChunkPos, (i32, i32, i32)) {
        let chunk_index = ChunkPos(
            self.0.div_euclid(Chunk::CHUNK_SIZE as i32),
            self.1.div_euclid(Chunk::CHUNK_SIZE as i32),
            self.2.div_euclid(Chunk::CHUNK_SIZE as i32),
        );
        let within_chunk_pos = (
            self.0.rem_euclid(Chunk::CHUNK_SIZE as i32),
            self.1.rem_euclid(Chunk::CHUNK_SIZE as i32),
            self.2.rem_euclid(Chunk::CHUNK_SIZE as i32),
        );

        (chunk_index, within_chunk_pos)
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air = 0,
    Dirt,
    Stone,
}

#[derive(Debug)]
pub struct Chunk {
    pub chunk_pos: ChunkPos, // Position of chunk in chunk space
    pub world_pos: WorldPos, // Position of corner block in world space
    pub blocks: [[[BlockType; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE], // Block type IDs
}

impl Chunk {
    pub const CHUNK_SIZE: usize = 16;

    const ADJACENT_OFFSETS: [[i32; 3]; 6] = [
        [-1, 0, 0],
        [1, 0, 0],
        [0, -1, 0],
        [0, 1, 0],
        [0, 0, -1],
        [0, 0, 1],
    ];

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
}

pub struct ChunkIter<'a> {
    chunk: &'a Chunk,
    index: usize,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= Chunk::CHUNK_SIZE.pow(3) {
            return None;
        }
        let (rem, x) = self.index.div_rem_euclid(&Chunk::CHUNK_SIZE);
        let (z, y) = rem.div_rem_euclid(&Chunk::CHUNK_SIZE);
        let block_pos = WorldPos(
            self.chunk.world_pos.0 + x as i32,
            self.chunk.world_pos.1 + y as i32,
            self.chunk.world_pos.2 + z as i32,
        );

        self.index += 1;

        Some(Block {
            world_pos: block_pos,
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
        fs::create_dir(folder).unwrap();
        self.chunks.iter().for_each(|(ChunkPos(x, y, z), chunk)| {
            let serialised = chunk
                .blocks
                .iter()
                .flatten()
                .flatten()
                .flat_map(|x| x.to_u16().unwrap().to_le_bytes())
                .collect::<Vec<_>>();

            let filename = folder.join(format!("{x}_{y}_{z}.chunk"));
            let mut writer =
                BufWriter::new(File::create_new(&filename).expect("Failed to create file"));
            writer.write_all(&serialised).unwrap();
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
                let chunk_pos = ChunkPos(x, y, z);

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
                    world_pos: chunk_pos.to_world_pos(),
                    chunk_pos,
                    blocks,
                }
            })
            .fold(FxHashMap::default(), |mut hm, chunk| {
                hm.insert(chunk.chunk_pos.clone(), chunk);
                hm
            });

        unimplemented!();
        let chunk_gen = ChunkGenerator::new(Perlin::new(42, 3, 0.5, 2., 1. / 64.));

        World {
            chunks,
            generator: chunk_gen,
        }
    }

    pub fn default() -> Self {
        let chunk_gen = ChunkGenerator::new(Perlin::new(42, 3, 0.5, 2., 1. / 64.));

        Self {
            chunks: Default::default(),
            generator: chunk_gen,
        }
    }

    /// Check if the given block has any side that isn't surrounded
    pub fn is_block_exposed(&self, pos: &WorldPos) -> bool {
        Chunk::ADJACENT_OFFSETS.iter().any(|o| {
            let (chunk_pos, within_chunk_pos) =
                WorldPos(pos.0 + o[0], pos.1 + o[1], pos.2 + o[2]).to_chunk_offset();

            if let Some(chunk) = self.chunks.get(&chunk_pos) {
                // If adjacent block is air, it's exposed
                *chunk.get_block(within_chunk_pos) == BlockType::Air
            } else {
                // If adjacent chunk hasn't been generated yet, it's exposed.
                true
            }
        })
    }

    pub fn get_or_generate_chunk(&mut self, pos: &ChunkPos) -> &Chunk {
        self.chunks
            .entry(pos.clone())
            .or_insert_with(|| self.generator.generate_chunk(pos.to_world_pos()))
    }
}
