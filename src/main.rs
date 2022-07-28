use classicl::{client, server::*, ClientController, Packet};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::terrain::{Terrain, TerrainNoise};

mod terrain;

const PLAYER_HEIGHT: i16 = 51 * 2;

#[tokio::main]
async fn main() {
    let mut server = classicl::Server::new("0.0.0.0:25565").await.unwrap();

    let pdb: Arc<Mutex<HashMap<i8, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    let pq = Arc::new(Mutex::new(HashMap::new()));

    println!("Generating Terrain...");
    let terrain = Arc::new(Mutex::new(Terrain::new((1024, 32, 1024))));
    println!("done.");

    let players = pq.clone();
    let map = terrain.clone();
    let spawn_point = (
        to_fixed_point(10.0),
        to_fixed_point(TerrainNoise::new().height(10, 10)) + PLAYER_HEIGHT,
        to_fixed_point(10.0),
    );
    server
        .on_client_connected(move |_s, id, c| {
            let mut players = players.lock().unwrap();
            players.insert(id, c.clone());

            c.write_packet(&Message {
                player_id: 0,
                message: "Welcome".to_string(),
            })
            .unwrap();
        })
        .await;

    let players = pdb.clone();
    let queue = pq.clone();
    server
        .on_player_identification(move |id, p| {
            let mut players = players.lock().unwrap();
            if let Some(c) = queue.lock().unwrap().remove(&id) {
                let player = Player {
                    c: c.clone(),
                    player_name: p.username.to_string(),
                    x: spawn_point.0,
                    y: spawn_point.1,
                    z: spawn_point.2,
                    yaw: 0,
                    pitch: 0,
                };

                c.write_packet(&LevelInitialize {}).unwrap();
                let mut buf = vec![];
                for i in &map.lock().unwrap().to_chunks() {
                    buf.push(LevelDataChunk::ID);
                    buf.append(&mut classicl::to_bytes(i).unwrap());
                }
                c.write_bytes(buf);

                c.write_packet(&LevelFinalize {
                    x_size: 1024,
                    y_size: 32,
                    z_size: 1024,
                })
                .unwrap();

                for (pid, p) in players.iter() {
                    p.c.write_packet(&player.to_spawn(id)).unwrap();
                    c.write_packet(&p.to_spawn(*pid)).unwrap();
                }
                c.write_packet(&player.to_spawn(-1)).unwrap();
                players.insert(id, player);
            }
        })
        .await;

    let players = pdb.clone();
    let map = terrain.clone();
    server
        .on_set_block(move |_, p| {
            let block_type = if p.mode == 0x00 {
                terrain::Blocks::Air as u8
            } else {
                p.block_type
            };
            map.lock().unwrap().set_block(p.x, p.y, p.z, block_type);
            for (_, player) in players.lock().unwrap().iter_mut() {
                player
                    .c
                    .write_packet(&SetBlock {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                        block_type,
                    })
                    .unwrap();
            }
        })
        .await;

    let players = pdb.clone();
    server
        .on_position_orientation(move |id, p| {
            let mut players = players.lock().unwrap();
            let mut mplayer = None;
            if let Some(player) = players.get_mut(&id) {
                player.set_pos_ori(p);
                mplayer = Some(player.clone())
            }
            if let Some(player) = mplayer {
                for (i, p) in players.iter_mut() {
                    if *i != id {
                        p.c.write_packet(&player.to_pos_ori_upd(id)).unwrap();
                    }
                }
            }
        })
        .await;

    let players = pdb.clone();
    let queue = pq.clone();

    server
        .on_client_disconnected(move |id| {
            let _ = players.lock().unwrap().remove(&id);
            let _ = queue.lock().unwrap().remove(&id);
            for (_, p) in players.lock().unwrap().iter_mut() {
                p.c.write_packet(&DespawnPlayer { player_id: id }).unwrap();
            }
        })
        .await;
    tokio::join!(server.run());
}

#[derive(Clone)]
pub struct Player {
    pub c: ClientController,
    pub player_name: String,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
}

impl Player {
    pub fn to_spawn(&self, id: i8) -> SpawnPlayer {
        SpawnPlayer {
            player_id: id,
            player_name: self.player_name.to_string(),
            x: self.x,
            y: self.y,
            z: self.z,
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }

    // ClassiCube does not understand PositionOrientationUpdate
    // maybe this is not meant to be used
    pub fn to_pos_ori_upd(&self, id: i8) -> client::PositionOrientation {
        client::PositionOrientation {
            player_id: id.to_be_bytes()[0],
            x: self.x,
            y: self.y,
            z: self.z,
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }

    pub fn set_pos_ori(&mut self, p: &client::PositionOrientation) {
        self.x = p.x;
        self.y = p.y;
        self.z = p.z;
        self.yaw = p.yaw;
        self.pitch = p.pitch;
    }
}

fn to_fixed_point(v: f64) -> i16 {
    (v * 32.0).round() as i16
}
