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

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct PlayerIdentification {
    pub protocol_version: u8,
    pub username: String,
    pub verification_key: String,
    pub unused: u8,
}

impl Packet for PlayerIdentification {
    const ID: u8 = 0x00;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct SetBlock {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub mode: u8,
    pub block_type: u8,
}

impl Packet for SetBlock {
    const ID: u8 = 0x05;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct PositionOrientation {
    pub player_id: u8,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
}

impl Packet for PositionOrientation {
    const ID: u8 = 0x08;
}

#[derive(Default, Debug, FixedSize, Serialize, Deserialize)]
pub struct Message {
    pub unused: u8,
    pub message: String,
}

impl Packet for Message {
    const ID: u8 = 0x0d;
}
