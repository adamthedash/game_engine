use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use glob::glob;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{Euclid, FromPrimitive, ToPrimitive};

use crate::{
    block::Block,
    world_gen::{ChunkGenerator, Perlin},
};

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air = 0,
    Dirt,
    Stone,
}

#[derive(Debug)]
pub struct Chunk {
    pub pos: (i32, i32, i32), // Position of corner block
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
            pos: 0,
        }
    }

    /// Check if the given block has any side that isn't surrounded
    pub fn is_block_exposed(&self, pos: (i32, i32, i32)) -> bool {
        let chunk_edges = [0, Self::CHUNK_SIZE as i32 - 1];
        let within_chunk_pos = [
            pos.0 % Self::CHUNK_SIZE as i32,
            pos.1 % Self::CHUNK_SIZE as i32,
            pos.2 % Self::CHUNK_SIZE as i32,
        ];
        // Edges of chunk always exposed
        if within_chunk_pos.iter().any(|p| chunk_edges.contains(p)) {
            return true;
        }

        // Chunks that are surrounded on all sides are not exposed
        if Self::ADJACENT_OFFSETS.iter().all(|x| {
            self.blocks[(within_chunk_pos[0] + x[0]) as usize]
                [(within_chunk_pos[1] + x[1]) as usize][(within_chunk_pos[2] + x[2]) as usize]
                != BlockType::Air
        }) {
            return false;
        }

        // By default a block is exposed
        true
    }
}

pub struct ChunkIter<'a> {
    chunk: &'a Chunk,
    pos: usize,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= Chunk::CHUNK_SIZE.pow(3) {
            return None;
        }
        let (rem, x) = self.pos.div_rem_euclid(&Chunk::CHUNK_SIZE);
        let (z, y) = rem.div_rem_euclid(&Chunk::CHUNK_SIZE);
        let block_pos = (
            self.chunk.pos.0 + x as i32,
            self.chunk.pos.1 + y as i32,
            self.chunk.pos.2 + z as i32,
        );

        self.pos += 1;

        Some(Block {
            world_pos: block_pos,
            block_type: self.chunk.blocks[x][y][z],
        })
    }
}

/// All of the world data
pub struct World {
    // Generated chunks
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
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
        self.chunks.iter().for_each(|((x, y, z), chunk)| {
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
                    pos: (x, y, z),
                    blocks,
                }
            })
            .fold(HashMap::new(), |mut hm, chunk| {
                hm.insert(chunk.pos, chunk);
                hm
            });

        World { chunks }
    }

    pub fn default() -> Self {
        let mut chunks = HashMap::new();

        let chunk_gen = ChunkGenerator::new(Perlin::new(42, 3, 2., 2., 1. / 16.));

        for i in 0..8 {
            let x = i * Chunk::CHUNK_SIZE;
            for j in 0..8 {
                let z = j * Chunk::CHUNK_SIZE;
                chunks
                    .entry((x as i32, 0, z as i32))
                    .insert_entry(chunk_gen.generate_chunk((x as i32, 0, z as i32)));
            }
        }

        Self { chunks }
    }
}
