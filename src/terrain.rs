pub enum Blocks {
    Air = 0,
    Stone = 1,
    Grass = 2,
    Dirt = 3,
    /*
    Cobblestone = 4,
    Wood = 5,
    Sapling = 6,
    Bedrock = 7,
    Water = 8,
    StillWater = 9,
    Lava = 10,
    StillLava = 11,
    Sand = 12,
    Gravel = 13,
    GoldOre = 14,
    IronOre = 15,
    CoalOre = 16,
    Log = 17,
    Leaves = 18,
    Sponge = 19,
    Glass = 20,
    Red = 21,
    Orange = 22,
    Yellow = 23,
    Lime = 24,
    Green = 25,
    Teal = 26,
    Aqua = 27,
    Cyan = 28,
    Blue = 29,
    Indigo = 30,
    Violet = 31,
    Magenta = 32,
    Pink = 33,
    Black = 34,
    Gray = 35,
    White = 36,
    Dandelion = 37,
    Rose = 38,
    BrownMushroom = 39,
    RedMushroom = 40,
    Gold = 41,
    Iron = 42,
    DoubleSlab = 43,
    Slab = 44,
    Brick = 45,
    TNT = 46,
    Bookshelf = 47,
    MossyRocks = 48,
    Obsidian = 49,
    CobblestoneSlab = 50,
    Rope = 51,
    Sandstone = 52,
    Snow = 53,
    Fire = 54,
    LightPink = 55,
    ForestGreen = 56,
    Brown = 57,
    DeepBlue = 58,
    Turquoise = 59,
    Ice = 60,
    CeramicTile = 61,
    Magma = 62,
    Pillar = 63,
    Crate = 64,
    StoneBrick = 65,
    */
}

use std::io::Write;

use classicl::server::LevelDataChunk;
use flate2::{Compression, write::GzEncoder as Enc};
use noise::{Add, Constant, NoiseFn, ScalePoint, SuperSimplex, ScaleBias};

pub struct TerrainNoise {
    height: SuperSimplex,
    caves: SuperSimplex,
}

impl TerrainNoise {
    pub fn new() -> Self {
        Self {
            height: SuperSimplex::new(),
            caves: SuperSimplex::new(),
        }
    }

    pub fn height(&self, x: i16, y: i16) -> f64 {
        let mut noise = ScaleBias::new(&self.height);
        noise.scale = 15.0;
        let h = Constant::new(20.0);
        let noise = Add::new(&noise, &h);
        let mut noise = ScalePoint::new(noise);
        noise.x_scale = 0.02;
        noise.y_scale = 0.02;
        noise.z_scale = 0.02;
        noise.get([x as f64, y as f64])
    }

    pub fn cave(&self, x: i16, y: i16, z: i16) -> f64 {
        let mut noise = ScalePoint::new(self.caves);
        noise.x_scale = 0.1;
        noise.y_scale = 0.1;
        noise.z_scale = 0.1;
        noise.get([x as f64, y as f64, z as f64])
    }
}

pub struct Terrain {
    size: (i16, i16, i16),
    inner: Vec<u8>,
}

impl Terrain {
    pub fn new(size: (i16, i16, i16)) -> Self {
        Self {
            size,
            inner: Self::generate(size),
        }
    }

    fn generate(size: (i16, i16, i16)) -> Vec<u8> {
        let noise = TerrainNoise::new();
        let (x, y, z) = size;
        let mut buf = vec![];
        for y in 0..y {
            for x in 0..x {
                for z in 0..z {
                    let h = noise.height(x, z);
                    if y as f64 > h {
                        buf.push(Blocks::Air as u8);
                    } else {
                        if noise.cave(x, y, z) > 0.005 {
                            if h.floor() as i16 - y > 5 {
                                buf.push(Blocks::Stone as u8);
                            } else if h.floor() as i16 - y > 0 {
                                buf.push(Blocks::Dirt as u8)
                            } else {
                                buf.push(Blocks::Grass as u8)
                            }
                        } else {
                            buf.push(Blocks::Air as u8);
                        }
                    }
                }
            }
        }
        buf
    }

    pub fn to_chunks(&self) -> Vec<LevelDataChunk> {
        let mut e = Enc::new(Vec::new(), Compression::fast());
        let size: [u8; 4] = (self.size.0 as u32 * self.size.1 as u32 * self.size.2 as u32).to_be_bytes();
        e.write_all(&size).unwrap();
        e.write_all(&self.inner).unwrap();
        let data = e.finish().unwrap();
        let mut bytes_sent = 0;
        let total = data.len();
        data.chunks(1024)
            .map(|x| {
                bytes_sent += x.len();
                let percent_complete = ((bytes_sent as f32 / total as f32) * 100.0).floor() as u8;
                LevelDataChunk {
                    chunk_length: x.len() as i16,
                    chunk_data: Vec::from(x),
                    percent_complete,
                }
            })
            .collect()
    }

    pub fn set_block(&mut self, x: i16, y: i16, z: i16, t: u8) {
        let (x_size, _, z_size) = self.size;
        let index = x as usize + x_size as usize * (z as usize + z_size as usize * y as usize);
        if let Some(v) = self.inner.get_mut(index) {
            *v = t;
        }
    }
}
