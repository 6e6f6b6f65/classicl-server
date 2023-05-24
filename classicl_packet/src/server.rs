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

use crate::Packet;
use classicl_serde::FixedSize;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct ServerIdentification {
    pub protocol_version: u8,
    pub server_name: String,
    pub server_motd: String,
    pub user_type: u8,
}

impl Packet for ServerIdentification {
    const ID: u8 = 0x00;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct Ping {}

impl Packet for Ping {
    const ID: u8 = 0x01;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct LevelInitialize {}

impl Packet for LevelInitialize {
    const ID: u8 = 0x02;
}

#[serde_as]
#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct LevelDataChunk {
    pub chunk_length: i16,
    #[serde_as(as = "Bytes")]
    pub chunk_data: Vec<u8>,
    pub percent_complete: u8,
}

impl Packet for LevelDataChunk {
    const ID: u8 = 0x03;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct LevelFinalize {
    pub x_size: i16,
    pub y_size: i16,
    pub z_size: i16,
}

impl Packet for LevelFinalize {
    const ID: u8 = 0x04;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct SetBlock {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub block_type: u8,
}

impl Packet for SetBlock {
    const ID: u8 = 0x06;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct SpawnPlayer {
    pub player_id: i8,
    pub player_name: String,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
}

impl Packet for SpawnPlayer {
    const ID: u8 = 0x07;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct PositionOrientationTeleport {
    pub player_id: i8,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
}

impl Packet for PositionOrientationTeleport {
    const ID: u8 = 0x08;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct PositionOrientationUpdate {
    pub player_id: i8,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
}

impl Packet for PositionOrientationUpdate {
    const ID: u8 = 0x09;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub player_id: i8,
    pub change_x: i16,
    pub change_y: i16,
    pub change_z: i16,
}

impl Packet for PositionUpdate {
    const ID: u8 = 0x0a;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct OrientationUpdate {
    pub player_id: i8,
    pub yaw: u8,
    pub pitch: u8,
}

impl Packet for OrientationUpdate {
    const ID: u8 = 0x0b;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct DespawnPlayer {
    pub player_id: i8,
}

impl Packet for DespawnPlayer {
    const ID: u8 = 0x0c;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct Message {
    pub player_id: i8,
    pub message: String,
}

impl Packet for Message {
    const ID: u8 = 0x0d;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct DisconnectPlayer {
    pub disconnect_reason: String,
}

impl Packet for DisconnectPlayer {
    const ID: u8 = 0x0e;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct UpdateUserType {
    pub user_type: u8,
}

impl Packet for UpdateUserType {
    const ID: u8 = 0x0f;
}
