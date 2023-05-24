/* This file is part of classicl-server.
 *
 * classicl-server is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

pub mod blocks;

const CAVE_THRESHOLD: f64 = 0.3;

use std::io::Write;

use classicl::{server::LevelDataChunk, Serialize};
use flate2::{write::GzEncoder as Enc, Compression};
use noise::{Abs, Add, Constant, Fbm, NoiseFn, ScaleBias, ScalePoint, SuperSimplex, Worley};
use serde::Deserialize;

use crate::PLAYER_HEIGHT;

use crate::to_fixed_point;

pub struct TerrainNoise {
    h: Constant,
    height: SuperSimplex,
    caves: Fbm,
    ores: SuperSimplex,
    trees: SuperSimplex,
    tree_height: Worley,
    roses: SuperSimplex,
    leaves: Worley,
}

impl TerrainNoise {
    pub fn new(h: f64) -> Self {
        let mut caves = Fbm::new();
        caves.octaves = 1;
        caves.lacunarity = 1.0;
        Self {
            h: Constant::new(h),
            height: SuperSimplex::new(),
            caves,
            ores: SuperSimplex::new(),
            trees: SuperSimplex::new(),
            tree_height: Worley::new(),
            roses: SuperSimplex::new(),
            leaves: Worley::new(),
        }
    }

    pub fn height(&self, x: i16, y: i16) -> f64 {
        let mut noise = ScaleBias::new(&self.height);
        noise.scale = 15.0;
        let noise = Add::new(&noise, &self.h);
        let mut noise = ScalePoint::new(noise);
        noise.x_scale = 0.02;
        noise.y_scale = 0.02;
        noise.z_scale = 0.02;
        noise.get([x as f64, y as f64])
    }

    pub fn cave(&self, x: i16, y: i16, z: i16) -> f64 {
        let mut noise = ScalePoint::new(&self.caves);
        noise.x_scale = 0.125;
        noise.y_scale = 0.125;
        noise.z_scale = 0.125;
        noise.get([x as f64, y as f64, z as f64])
    }

    pub fn ore(&self, x: i16, y: i16, z: i16) -> f64 {
        self.ores.get([x as f64, y as f64, z as f64])
    }

    pub fn ground(&self, x: i16, y: i16, z: i16, tree_pos: &mut Vec<Decoration>) {
        let h = self.height(x, z).floor();
        let mut trees = ScalePoint::new(&self.trees);
        trees.x_scale = 10.0;
        trees.y_scale = 10.0;
        trees.z_scale = 10.0;
        if h as i16 + 1 == y {
            if trees.get([x as f64, z as f64, h]) > 0.8 {
                let mut tree_height = ScaleBias::new(&self.tree_height);
                tree_height.scale = 7.0;
                let tree_height = Abs::new(&tree_height);
                let tree_h = tree_height.get([x as f64, z as f64]);
                let tree_h = tree_h as i16;
                let h = h as i16;
                if h + 1 == y {
                    tree_pos.push(Decoration {
                        x,
                        y,
                        z,
                        t: DecorationType::Tree(tree_h),
                    });
                }
            } else if self.roses.get([x as f64, z as f64]) > 0.7 {
                tree_pos.push(Decoration {
                    x,
                    y,
                    z,
                    t: DecorationType::Rose,
                });
            }
        }
    }

    pub fn leaves(&self, x: i16, y: i16, z: i16) -> f64 {
        self.leaves.get([x.into(), y.into(), z.into()])
    }
}

#[derive(Serialize, Deserialize)]
pub struct Terrain {
    pub size: (i16, i16, i16),
    pub spawn_point: (i16, i16, i16),
    inner: Vec<u8>,
}

impl Terrain {
    pub fn new(size: (i16, i16, i16), height: f64) -> Self {
        Self {
            size,
            spawn_point: (
                to_fixed_point(10.0),
                to_fixed_point(TerrainNoise::new(height).height(10, 10)) + PLAYER_HEIGHT,
                to_fixed_point(10.0),
            ),
            inner: Self::generate(size, height),
        }
    }

    fn generate(size: (i16, i16, i16), height: f64) -> Vec<u8> {
        let mut tree_pos = vec![];
        let noise = TerrainNoise::new(height);
        let (x, y, z) = size;
        let mut buf = vec![];

        for y in 0..y {
            for z in 0..z {
                for x in 0..x {
                    let h = noise.height(x, z);
                    if y as f64 > h {
                        noise.ground(x, y, z, &mut tree_pos);
                        buf.push(blocks::AIR);
                    } else if noise.cave(x, y, z) > CAVE_THRESHOLD {
                        buf.push(blocks::AIR);
                    } else if h.floor() as i16 - y > 5 {
                        let ore = noise.ore(x, y, z);
                        if ore > 0.9 {
                            buf.push(blocks::GOLD_ORE)
                        } else if ore > 0.8 {
                            buf.push(blocks::IRON_ORE)
                        } else if ore > 0.7 {
                            buf.push(blocks::COAL_ORE)
                        } else {
                            buf.push(blocks::STONE)
                        }
                    } else if h.floor() as i16 - y > 0 {
                        buf.push(blocks::DIRT)
                    } else {
                        buf.push(blocks::GRASS)
                    }
                }
            }
        }

        for i in tree_pos.into_iter() {
            let (x_pos, y_pos, z_pos) = (i.x, i.y, i.z);
            if let Some(index) = index(size.0, size.2, x_pos, y_pos - 1, z_pos) {
                if let Some(&block) = buf.get(index) {
                    if block == blocks::AIR {
                        continue;
                    }
                }
            }
            match i.t {
                DecorationType::Rose => {
                    if let Some(index) = index(size.0, size.2, x_pos, y_pos, z_pos) {
                        if let Some(block) = buf.get_mut(index) {
                            *block = blocks::ROSE;
                        }
                    }
                }
                DecorationType::Tree(tree_h) => {
                    for y in 0..=tree_h {
                        if let Some(index) = index(size.0, size.2, x_pos, y + y_pos, z_pos) {
                            if let Some(block) = buf.get_mut(index) {
                                *block = blocks::LOG;
                            }
                        }
                    }
                    for y in -2..=1 {
                        for x in -2..=2 {
                            for z in -2..=2 {
                                if let Some(index) =
                                    index(size.0, size.2, x + x_pos, y + tree_h + y_pos, z + z_pos)
                                {
                                    if let Some(block) = buf.get_mut(index) {
                                        if y < 0 {
                                            if (x == 2 || x == -2) && (z == 2 || z == -2) {
                                                if noise.leaves(
                                                    x + x_pos,
                                                    y + tree_h + y_pos,
                                                    z + z_pos,
                                                ) > 0.3
                                                    && *block == blocks::AIR
                                                {
                                                    *block = blocks::LEAVES;
                                                }
                                            } else if *block == blocks::AIR {
                                                *block = blocks::LEAVES;
                                            }
                                        } else if (x < 2 && x > -2) && (z < 2 && z > -2) {
                                            if (x == 1 || x == -1) && (z == 1 || z == -1) {
                                                if y == 0
                                                    && noise.leaves(
                                                        x + x_pos,
                                                        y + tree_h + y_pos,
                                                        z + z_pos,
                                                    ) > 0.3
                                                    && *block == blocks::AIR
                                                {
                                                    *block = blocks::LEAVES;
                                                }
                                            } else if *block == blocks::AIR {
                                                *block = blocks::LEAVES;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        buf
    }

    pub fn to_chunks(&self) -> Vec<LevelDataChunk> {
        let mut e = Enc::new(Vec::new(), Compression::fast());
        let size: [u8; 4] =
            (self.size.0 as u32 * self.size.1 as u32 * self.size.2 as u32).to_be_bytes();
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
        if let Some(index) = index(x_size, z_size, x, y, z) {
            if let Some(v) = self.inner.get_mut(index) {
                *v = t;
            }
        }
    }
}

fn index(x_size: i16, z_size: i16, x: i16, y: i16, z: i16) -> Option<usize> {
    let index = x as i64 + x_size as i64 * (z as i64 + z_size as i64 * y as i64);

    if index > usize::MAX as i64 {
        Some(index as usize)
    } else {
        None
    }
}

enum DecorationType {
    Rose,
    Tree(i16),
}

pub struct Decoration {
    x: i16,
    y: i16,
    z: i16,
    t: DecorationType,
}
