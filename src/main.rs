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
use tokio::{select, sync::oneshot, time};

use crate::{cli::Cli, commands::Command, terrain::Terrain};
use clap::Parser;

mod cli;
mod commands;
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
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&cli.data)
        .unwrap();
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

    info!("Terrain ready.");

    let is_changed = Arc::new(Mutex::new(false));

    let opt = cli.clone();
    server
        .on_server_full(move || classicl::server::DisconnectPlayer {
            disconnect_reason: format!("&cSorry, {} &cis full right now.", opt.name),
        })
        .await;

    let handler = server.on_client_connected().unwrap();
    let players = pq.clone();
    let opt = cli.clone();
    tokio::spawn(async move {
        let mut handler = handler.await.unwrap();
        while let Some(data) = handler.get().await {
            data.client.write_packet(&ServerIdentification {
                protocol_version: 0x07,
                server_name: opt.name.clone(),
                server_motd: opt.motd.clone(),
                user_type: 0x00,
            })
            .unwrap();

            let mut players = players.lock().unwrap();
            let (tx, rx) = oneshot::channel();
            players.insert(data.id, (data.client.clone(), tx));
            let c = data.client.clone();
            tokio::spawn(async move {
                select! {
                    _ = rx => (),
                    _ = time::sleep(Duration::from_secs(2)) => {
                        c.disconnect(Some(&DisconnectPlayer {
                            disconnect_reason: "Identification timeout".into()
                        }));
                    }
                }
            });
        }
    });

    let handler = server.on_player_identification().unwrap();
    let players = pdb.clone();
    let queue = pq.clone();
    let map = terrain.clone();
    tokio::spawn(async move {
        let mut handler = handler.await.unwrap();
        while let Some(data) = handler.get().await {
            let mut players = players.lock().unwrap();
            if let Some((c, tx)) = queue.lock().unwrap().remove(&data.id) {
                tx.send(()).unwrap();
                let spawn_point = map.lock().unwrap().spawn_point;
                let player = Player {
                    c: c.clone(),
                    player_name: data.data.username.to_string(),
                    x: spawn_point.0,
                    y: spawn_point.1,
                    z: spawn_point.2,
                    yaw: 0,
                    pitch: 0,
                };

                info!("{} identified as {}", data.id, data.data.username.trim());

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
                    p.c.write_packet(&player.to_spawn(data.id)).unwrap();
                    c.write_packet(&p.to_spawn(*pid)).unwrap();
                }
                c.write_packet(&player.to_spawn(-1)).unwrap();
                players.insert(data.id, player);
            }
        }
    });

    let handler = server.on_set_block().unwrap();
    let players = pdb.clone();
    let map = terrain.clone();
    let changed = is_changed.clone();
    tokio::spawn(async move {
        let mut handler = handler.await.unwrap();
        while let Some(data) = handler.get().await {
            *changed.lock().unwrap() = true;
            if players.lock().unwrap().get(&data.id).is_some() {
                let block_type = if data.data.mode == 0x00 {
                    terrain::blocks::AIR
                } else {
                    data.data.block_type
                };
                map.lock().unwrap().set_block(data.data.x, data.data.y, data.data.z, block_type);
                for (_, player) in players.lock().unwrap().iter_mut() {
                    player
                        .c
                        .write_packet(&SetBlock {
                            x: data.data.x,
                            y: data.data.y,
                            z: data.data.z,
                            block_type,
                        })
                        .unwrap();
                }
            }
        }
    });

    let handler = server.on_position_orientation().unwrap();
    let players = pdb.clone();
    tokio::spawn(async move {
        let mut handler = handler.await.unwrap();
        while let Some(data) = handler.get().await {
            let mut players = players.lock().unwrap();
            let mut mplayer = None;
            if let Some(player) = players.get_mut(&data.id) {
                player.set_pos_ori(&data.data);
                mplayer = Some(player.clone())
            }
            if let Some(player) = mplayer {
                for (i, p) in players.iter_mut() {
                    if *i != data.id {
                        p.c.write_packet(&player.to_pos_ori_upd(data.id)).unwrap();
                    }
                }
            }
        }
    });

    let handler = server.on_message().unwrap();
    let players = pdb.clone();
    tokio::spawn(async move {
        let mut handler = handler.await.unwrap();
        while let Some(data) = handler.get().await {
            let mut players = players.lock().unwrap();

            if let Some(player) = players.get(&data.id) {
                let message: &str = data.data.message.trim();

                // Commands
                if message.starts_with('/') {
                    match Command::from_str(&message[1..]) {
                        Ok(cmd) => match cmd {
                            Command::Tp(other_p) => {
                                if let Some((o_id, other_p)) = players
                                    .iter()
                                    .find(|(_, p)| p.player_name.trim() == &other_p)
                                {
                                    info!("{} teleported to {o_id}", data.id);
                                    player
                                        .c
                                        .write_packet(&PositionOrientationTeleport {
                                            player_id: -1,
                                            x: other_p.x,
                                            y: other_p.y,
                                            z: other_p.z,
                                            yaw: other_p.yaw,
                                            pitch: 0,
                                        })
                                        .unwrap();
                                } else {
                                    debug!("{} tried to teleport to {other_p}", data.id);
                                    player.write_message(format!(
                                        "&cCould not find player `{}`",
                                        other_p
                                    ));
                                }
                            }
                        },
                        Err(e) => {
                            debug!("{} tried to execute `{message}`", data.id);
                            match e {
                                commands::CommandError::NoCommand => player
                                    .write_message(format!("&c`{}` is not a command", message)),
                                commands::CommandError::CommandNotKnown => {
                                    player.write_message(format!("&c`{}` is not known", message))
                                }
                                commands::CommandError::TooManyArguments => player.write_message(
                                    format!("&c`{}` has too many arguments", message),
                                ),
                                commands::CommandError::NotEnoughArguments => player.write_message(
                                    format!("&c`{}` has not enough arguments", message),
                                ),
                            }
                        }
                    }
                } else {
                    let mut message =
                        format!("&7{}:&f {}", player.player_name.trim(), data.data.message.trim());
                    info!("{} wrote: {}", player.player_name.trim(), data.data.message.trim());
                    message.truncate(64);
                    for (_, p) in players.iter_mut() {
                        p.c.write_packet(&Message {
                            player_id: data.id,
                            message: message.clone(),
                        })
                        .unwrap();
                    }
                }
            }
        }
    });

    let handler = server.on_client_disconnected().unwrap();
    let players = pdb.clone();
    let queue = pq.clone();
    tokio::spawn(async move {
        let mut handler = handler.await.unwrap();
        while let Some(data) = handler.get().await {
            let _ = players.lock().unwrap().remove(&data.id);
            let _ = queue.lock().unwrap().remove(&data.id);
            for (_, p) in players.lock().unwrap().iter_mut() {
                p.c.write_packet(&DespawnPlayer { player_id: data.id }).unwrap();
            }
        }
    });

    let map = terrain.clone();
    let opt = cli.clone();
    let changed = is_changed.clone();
    tokio::spawn(async move {
        loop {
            time::sleep(Duration::from_secs(120)).await;
            let mut changed = changed.lock().unwrap();
            if *changed {
                debug!("Trying to save the map.");
                *changed = false;
                save_map(opt.clone(), map.clone());
            } else {
                debug!("Map not changed. Save discarded")
            }
        }
    });

    let map = terrain.clone();
    let opt = cli.clone();
    let ctrl_c = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Saving map and stopping server now.");
        save_map(opt, map);
    });

    let opt = cli.clone();
    tokio::select! {
        _ = server.run(opt.limit) => (),
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

    pub fn write_message(&self, mut message: String) {
        message.truncate(64);
        self.c
            .write_packet(&Message {
                player_id: 0,
                message,
            })
            .unwrap();
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
