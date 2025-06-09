use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use glob::glob;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
enum BlockType {
    Air = 0,
    Dirt,
}

struct Chunk {
    pos: (i32, i32, i32),                // Position of corner block
    blocks: [[[BlockType; 16]; 16]; 16], // Block type IDs
}

/// All of the world data
struct World {
    // Generated chunks
    chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl World {
    /// Save the world data to disk
    /// 1 chunk = 1 file, block types stored as a flat array
    pub fn save(&self, folder: &Path) {
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
                    .chunks_exact(2)
                    .map(|c| {
                        BlockType::from_u16(u16::from_le_bytes(c.try_into().unwrap())).unwrap()
                    })
                    .collect::<Vec<_>>()
                    .chunks_exact(16)
                    .map(|c| c.try_into().unwrap())
                    .collect::<Vec<[_; 16]>>()
                    .chunks_exact(16)
                    .map(|c| c.try_into().unwrap())
                    .collect::<Vec<[_; 16]>>()
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
}
