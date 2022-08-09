use classicl::{client, server::*, ClientController, Packet};
use log::{debug, info};
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time;

use crate::{cli::Cli, terrain::Terrain};
use clap::Parser;

mod cli;
mod terrain;

const PLAYER_HEIGHT: i16 = 51 * 2;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();
    let cli = Arc::new(Cli::parse());
    let mut server = classicl::Server::new(&cli.adress).await.unwrap();

    let pdb: Arc<Mutex<HashMap<i8, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    let pq = Arc::new(Mutex::new(HashMap::new()));

    let path = generate_path(&cli.data);
    let terrain = Arc::new(Mutex::new(if let Ok(file) = File::open(path) {
        info!("Loading Terrain...");
        bincode::deserialize_from(file).unwrap()
    } else {
        info!("Generating Terrain...");
        Terrain::new((cli.x_size, cli.y_size, cli.z_size), cli.height)
    }));

    let players = pq.clone();

    let opt = cli.clone();
    server
        .on_client_connected(move |_, id, c| {
            c.write_packet(&ServerIdentification {
                protocol_version: 0x07,
                server_name: opt.name.clone(),
                server_motd: opt.motd.clone(),
                user_type: 0x00,
            })
            .unwrap();
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
    let map = terrain.clone();
    server
        .on_player_identification(move |id, p| {
            let mut players = players.lock().unwrap();
            if let Some(c) = queue.lock().unwrap().remove(&id) {
                let spawn_point = map.lock().unwrap().spawn_point;
                let player = Player {
                    c: c.clone(),
                    player_name: p.username.to_string(),
                    x: spawn_point.0,
                    y: spawn_point.1,
                    z: spawn_point.2,
                    yaw: 0,
                    pitch: 0,
                };

                info!("{id} identified as {}", p.username.trim());

                let mut buf = vec![LevelInitialize::ID];
                {
                    buf.append(&mut (classicl::to_bytes(&LevelInitialize {}).unwrap()));
                    let map = map.lock().unwrap();
                    for i in map.to_chunks() {
                        buf.push(LevelDataChunk::ID);
                        buf.append(&mut classicl::to_bytes(i).unwrap());
                    }

                    buf.push(LevelFinalize::ID);
                    buf.append(
                        &mut classicl::to_bytes(&LevelFinalize {
                            x_size: map.size.0,
                            y_size: map.size.1,
                            z_size: map.size.2,
                        })
                        .unwrap(),
                    );
                }
                c.write_bytes(buf);

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
    server
        .on_message(move |id, m| {
            let mut players = players.lock().unwrap();

            if let Some(player) = players.get(&id) {
                let mut message =
                    format!("&7{}:&f {}", player.player_name.trim(), m.message.trim());
                info!("{} wrote: {}", player.player_name.trim(), m.message.trim());
                message.truncate(64);
                for (_, p) in players.iter_mut() {
                    p.c.write_packet(&Message {
                        player_id: id,
                        message: message.clone(),
                    })
                    .unwrap();
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

    let players = pdb.clone();
    let map = terrain.clone();
    let opt = cli.clone();
    tokio::spawn(async move {
        loop {
            time::sleep(Duration::from_secs(300)).await;
            if !players.lock().unwrap().is_empty() {
                debug!("Trying to save the map.");
                save_map(opt.clone(), map.clone());
            }
        }
    });

    let map = terrain.clone();
    let cli = cli.clone();
    let ctrl_c = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Saving map and stopping server now.");
        save_map(cli, map);
    });

    tokio::select! {
        _ = server.run() => (),
        _ = ctrl_c => (),
    }
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

fn save_map(cli: Arc<Cli>, map: Arc<Mutex<Terrain>>) {
    let map = map.lock().unwrap();
    let data = bincode::serialize(&*map).unwrap();
    let path = generate_path(&cli.data);

    let mut file = File::create(path).unwrap();
    file.write_all(&data).unwrap();
}

fn generate_path(path: &PathBuf) -> PathBuf {
    let dir: &Path = path.as_path();
    dir.join("mapdata")
}
