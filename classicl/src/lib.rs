/* This file is part of classicl.
 *
 * classicl is free software: you can redistribute it and/or modify
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

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc::*, oneshot};

use classicl_packet::client::*;
pub use classicl_packet::{client, server, Packet};
pub use classicl_serde::{from_bytes, to_bytes};

use log::{debug, error, info, trace};
pub use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

pub use classicl_serde::FixedSize;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

type OnServerFull = Arc<Mutex<Box<dyn FnMut() -> server::DisconnectPlayer>>>;
type InitialHandleReceiver<T> = oneshot::Receiver<SignalHandle<T>>;
type InitialHandleSender<T> = oneshot::Sender<SignalHandle<T>>;
type InitialHandle<T> = (
    Option<InitialHandleSender<T>>,
    Option<InitialHandleReceiver<T>>,
);

pub struct Server {
    tcp: TcpListener,
    id_stack: Arc<Mutex<Vec<i8>>>,
    current_id: i8,
    on_client_connected: InitialHandle<OnClientConnected>,
    on_client_disconnected: InitialHandle<OnClientDisconnected>,
    on_player_identification: InitialHandle<OnPlayerIdentification>,
    on_set_block: InitialHandle<OnSetBlock>,
    on_position_orientation: InitialHandle<OnPositionOrientation>,
    on_message: InitialHandle<OnMessage>,
    on_server_full: OnServerFull,
}

impl Server {
    /// Creates a new Classicl server ready to be configured
    ///
    /// # Examples
    ///
    /// ```rust
    /// use classicl::Server;
    /// use classicl::server::DisconnectPlayer;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut server = Server::new("0.0.0.0:25565").await.unwrap();
    ///
    ///     server.on_client_connected(move |addr, id, ctrl| {
    ///         println!("{id} connected ({addr})");
    ///         ctrl.disconnect(Some(&DisconnectPlayer { disconnect_reason: "It works!".into() }));
    ///     }).await;
    ///
    ///     // Now you can try to connect to the server
    ///     server.run(None).await;
    /// }
    /// ```
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self, Box<dyn std::error::Error>> {
        let tcp = TcpListener::bind(addr).await?;
        info!("Server listening.");
        Ok(Self {
            tcp,
            id_stack: Arc::new(Mutex::new(Vec::with_capacity(127))),
            current_id: 0,
            on_client_connected: generate_initial_handle(oneshot::channel()),
            on_client_disconnected: generate_initial_handle(oneshot::channel()),
            on_player_identification: generate_initial_handle(oneshot::channel()),
            on_set_block: generate_initial_handle(oneshot::channel()),
            on_position_orientation: generate_initial_handle(oneshot::channel()),
            on_message: generate_initial_handle(oneshot::channel()),
            on_server_full: Arc::new(Mutex::new(Box::new(|| server::DisconnectPlayer {
                disconnect_reason: "".into(),
            }))),
        })
    }

    /// Runs the previously configured server instance, when no limit is given the maximum allowed
    /// number of clients is [`i8::MAX`]
    pub async fn run(&mut self, limit: Option<i8>) {
        let listener = &self.tcp;

        let (handle, on_client_connected) = SignalHandle::new();
        self.on_client_connected
            .0
            .take()
            .unwrap()
            .send(handle)
            .unwrap();

        let (handle, on_client_disconnected) = SignalHandle::new();
        self.on_client_disconnected
            .0
            .take()
            .unwrap()
            .send(handle)
            .unwrap();

        let (handle, on_player_identification) = SignalHandle::new();
        self.on_player_identification
            .0
            .take()
            .unwrap()
            .send(handle)
            .unwrap();

        let (handle, on_set_block) = SignalHandle::new();
        self.on_set_block.0.take().unwrap().send(handle).unwrap();

        let (handle, on_position_orientation) = SignalHandle::new();
        self.on_position_orientation
            .0
            .take()
            .unwrap()
            .send(handle)
            .unwrap();

        let (handle, on_message) = SignalHandle::new();
        self.on_message.0.take().unwrap().send(handle).unwrap();

        debug!("Starting server loop.");
        loop {
            let (socket, s) = {
                let result = listener.accept().await;

                match result {
                    Err(e) => {
                        error!("Cannot accept connection: {}", e);
                        continue;
                    }
                    Ok(v) => v,
                }
            };

            let id = if let Some(i) = self.id_stack.lock().await.pop() {
                i
            } else if self.current_id < i8::MAX
                && if let Some(limit) = limit {
                    self.current_id < limit
                } else {
                    true
                }
            {
                self.current_id += 1;
                self.current_id
            } else {
                let (_, mut write) = socket.into_split();
                write
                    .write_all(&[server::DisconnectPlayer::ID])
                    .await
                    .unwrap();
                write
                    .write_all(&to_bytes((self.on_server_full.lock().await)()).unwrap())
                    .await
                    .unwrap();
                continue;
            };
            info!("{} connected with id {}", s, id);
            let (send, recv) = mpsc::channel(16);
            let disconnect = CancellationToken::new();

            let ctrl = ClientController {
                sender: send,
                disconnect: disconnect.clone(),
            };
            on_client_connected
                .send(OnClientConnected {
                    addr: s,
                    id,
                    client: ctrl,
                })
                .await
                .unwrap();

            let id_stack = self.id_stack.clone();

            let on_client_disconnected = on_client_disconnected.clone();
            let on_player_identification = on_player_identification.clone();
            let on_set_block = on_set_block.clone();
            let on_position_orientation = on_position_orientation.clone();
            let on_client_message = on_message.clone();
            tokio::spawn(async move {
                if Self::client_loop(
                    socket,
                    recv,
                    id,
                    on_player_identification,
                    on_set_block,
                    on_position_orientation,
                    on_client_message,
                    disconnect,
                )
                .await
                .is_err()
                {
                    on_client_disconnected
                        .send(OnClientDisconnected { id })
                        .await
                        .unwrap();
                    id_stack.lock().await.push(id);
                    info!("{} disconnected.", id);
                }
            });
        }
    }

    async fn client_loop(
        socket: TcpStream,
        mut recv: Receiver<Vec<u8>>,
        id: i8,
        on_player_identification: Sender<OnPlayerIdentification>,
        on_set_block: Sender<OnSetBlock>,
        on_position_orientation: Sender<OnPositionOrientation>,
        on_message: Sender<OnMessage>,
        disconnect: CancellationToken,
    ) -> Result<()> {
        trace!("{id}'s client loop started.");
        let (mut reader, mut writer) = socket.into_split();

        let cancel = disconnect.clone();
        let write: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            let task = async {
                while let Some(p) = recv.recv().await {
                    writer.write_all(&p).await?;
                    trace!("writing some bytes to {id}");
                }
                Ok(())
            };
            tokio::select! {
                res = task => {
                    res
                },
                _ = async {
                    cancel.cancelled().await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                } => {
                    debug!("dropping {id}'s write half");
                    Err(anyhow::anyhow!("kicked"))
                }
            }
        });

        let cancel = disconnect.clone();
        let read: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            let task = async {
                loop {
                    let mut buf = [0u8];
                    reader.read_exact(&mut buf).await?;
                    match buf[0] {
                        PlayerIdentification::ID => {
                            trace!("PlayerIdentification received from {id}");
                            let mut buf = [0_u8; PlayerIdentification::SIZE];
                            reader.read_exact(&mut buf).await?;
                            let data: PlayerIdentification = from_bytes(&buf)?;

                            on_player_identification
                                .send(OnPlayerIdentification { id, data })
                                .await?;
                        }
                        client::SetBlock::ID => {
                            trace!("SetBlock received from {id}");
                            let mut buf = [0u8; client::SetBlock::SIZE];
                            reader.read_exact(&mut buf).await?;
                            let data: client::SetBlock = from_bytes(&buf)?;

                            on_set_block.send(OnSetBlock { id, data }).await?;
                        }
                        client::PositionOrientation::ID => {
                            trace!("PositionOrientation received from {id}");
                            let mut buf = [0u8; client::PositionOrientation::SIZE];
                            reader.read_exact(&mut buf).await?;
                            let data: client::PositionOrientation = from_bytes(&buf)?;

                            on_position_orientation
                                .send(OnPositionOrientation { id, data })
                                .await?;
                        }
                        client::Message::ID => {
                            trace!("Message received from {id}");
                            let mut buf = [0u8; client::Message::SIZE];
                            reader.read_exact(&mut buf).await?;
                            let data: client::Message = from_bytes(&buf)?;

                            on_message.send(OnMessage { id, data }).await?;
                        }
                        i => trace!("Unknown packet id ({i}) received from {id}"),
                    }
                }
            };
            tokio::select! {
                res = task => {
                    res
                },
                _ = async {
                    cancel.cancelled().await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                } => {
                    debug!("dropping {id}'s read half");
                    Err(anyhow::anyhow!("kicked"))
                }
            }
        });

        tokio::select! {
            read = read => {
                read.unwrap()
            }
            write = write => {
                write.unwrap()
            }
        }
    }

    /// Calls given function when a new client connects to the server. Provides a [`ClientController`]
    /// which can be cloned freely for later use and the id as a signed byte.
    pub fn on_client_connected(&mut self) -> Option<InitialHandleReceiver<OnClientConnected>> {
        self.on_client_connected.1.take()
    }

    /// Calls given function when client with specific id disconnected.
    pub fn on_client_disconnected(
        &mut self,
    ) -> Option<InitialHandleReceiver<OnClientDisconnected>> {
        self.on_client_disconnected.1.take()
    }

    /// Calls given function when a client (identified by id) wrote a [`client::PlayerIdentification`] Packet.
    pub fn on_player_identification(
        &mut self,
    ) -> Option<InitialHandleReceiver<OnPlayerIdentification>> {
        self.on_player_identification.1.take()
    }

    /// Calls given function when a client (identified by id) wrote a [`client::SetBlock`] Packet.
    pub fn on_set_block(&mut self) -> Option<InitialHandleReceiver<OnSetBlock>> {
        self.on_set_block.1.take()
    }

    /// Calls given function when a client (identified by id) wrote a [`client::PositionOrientation`] Packet.
    pub fn on_position_orientation(
        &mut self,
    ) -> Option<InitialHandleReceiver<OnPositionOrientation>> {
        self.on_position_orientation.1.take()
    }

    /// Calls given function when a client (identified by id) wrote a [`client::Message`] Packet.
    pub fn on_message(&mut self) -> Option<InitialHandleReceiver<OnMessage>> {
        self.on_message.1.take()
    }

    /// Calls given function when either the ids are exhausted (>= [`i8::MAX`]) or the given limit is reached.
    pub async fn on_server_full<F>(&mut self, f: F)
    where
        F: FnMut() -> server::DisconnectPlayer + 'static + Send,
    {
        *self.on_server_full.lock().await = Box::new(f);
    }
}

#[derive(Debug)]
pub struct SignalHandle<T> {
    rx: Receiver<T>,
}
impl<T> SignalHandle<T> {
    pub fn new() -> (Self, Sender<T>) {
        let (tx, rx) = channel(10);
        (Self { rx }, tx)
    }
    pub async fn get(&mut self) -> Option<T> {
        self.rx.recv().await
    }
}

#[derive(Clone, Debug)]
pub struct OnClientConnected {
    pub addr: SocketAddr,
    pub id: i8,
    pub client: ClientController,
}

#[derive(Clone, Debug)]
pub struct OnClientDisconnected {
    pub id: i8,
}
#[derive(Debug)]
pub struct OnPlayerIdentification {
    pub id: i8,
    pub data: PlayerIdentification,
}

#[derive(Debug)]
pub struct OnSetBlock {
    pub id: i8,
    pub data: SetBlock,
}

#[derive(Debug)]
pub struct OnPositionOrientation {
    pub id: i8,
    pub data: PositionOrientation,
}

#[derive(Debug)]
pub struct OnMessage {
    pub id: i8,
    pub data: client::Message,
}

/// Controls the connection to a specific client.
///
/// # Examples
///
/// ```rust
/// use classicl::Server;
/// use classicl::server::{DisconnectPlayer, ServerIdentification};
/// use std::collections::HashMap;
/// use std::sync::{Arc, Mutex};
///
/// #[tokio::main]
/// async fn main() {
///     let mut server = Server::new("0.0.0.0:25565").await.unwrap();
///
///     let players = Arc::new(Mutex::new(HashMap::new()));
///
///     let playersc = players.clone();
///     server.on_client_connected(move |_, id, ctrl| {
///         let mut players = playersc.lock().unwrap();
///         ctrl.write_packet(&ServerIdentification {
///             protocol_version: 0x07,
///             server_name: "Demo".into(),
///             server_motd: "Hello, World!".into(),
///             user_type: 0x00,
///         });
///
///         // ClientController is owned by this function so you can move it somewhere
///         // else to use it later.
///         players.insert(id, ctrl);
///     }).await;
///
///     server.on_player_identification(move |id, packet| {
///         let mut players = players.lock().unwrap();
///         if let Some(ctrl) = players.get(&id) {
///             ctrl.disconnect(Some(&DisconnectPlayer {
///                 disconnect_reason: format!("Hello {}. This is just a demo!", packet.username.trim())
///             }));
///         }
///     }).await;
///
///     server.run(None).await;
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ClientController {
    sender: mpsc::Sender<Vec<u8>>,
    disconnect: CancellationToken,
}

impl ClientController {
    /// Writes a packet into the buffer.
    ///
    /// Does not necessarily write packets in the given order. If you want to do that (e.g. for the map) use
    /// [`Self::write_bytes`].
    pub async fn write_packet<T: Serialize + Packet>(
        &self,
        p: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = vec![T::ID];
        buf.append(&mut to_bytes(p)?);
        let sender = self.sender.clone();
        trace!("Trying to send a packet with id {}", T::ID);
        tokio::spawn(async move { sender.send(buf).await.unwrap() });
        Ok(())
    }

    /// Writes bytes into the buffer.
    pub async fn write_bytes(&self, b: Vec<u8>) {
        let sender = self.sender.clone();
        trace!("Trying to send some bytes.");
        tokio::spawn(async move { sender.send(b).await.unwrap() })
            .await
            .unwrap();
    }

    /// Queues the client for disconnection. If reason given it will displayed when the player leaves the game.
    pub async fn disconnect(&self, reason: Option<&server::DisconnectPlayer>) {
        if let Some(reason) = reason {
            self.write_packet(reason).await.unwrap();
        }
        self.disconnect.cancel();
    }
}

fn generate_initial_handle<T>(
    channel: (
        oneshot::Sender<SignalHandle<T>>,
        oneshot::Receiver<SignalHandle<T>>,
    ),
) -> InitialHandle<T> {
    (Some(channel.0), Some(channel.1))
}
