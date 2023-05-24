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

use clap::Parser;
use std::path::PathBuf;

/// A Block Game Server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Adress to listen on
    #[clap(short, long, value_parser, default_value_t = String::from("0.0.0.0:25565"))]
    pub adress: String,

    /// x size
    #[clap(short, long, value_parser = clap::value_parser!(i16).range(1..), default_value_t = 128)]
    pub x_size: i16,

    /// y size
    #[clap(short, long, value_parser = clap::value_parser!(i16).range(1..), default_value_t = 32)]
    pub y_size: i16,

    /// x size
    #[clap(short, long, value_parser = clap::value_parser!(i16).range(1..), default_value_t = 128)]
    pub z_size: i16,

    /// Ground height
    #[clap(short, long, value_parser, default_value_t = 20.0)]
    pub terrain_height: f64,

    /// Data directory
    #[clap(short, long, value_parser, default_value_os_t = PathBuf::from("data"))]
    pub data: PathBuf,

    /// Server name shown when connecting
    #[clap(short, long, value_parser, default_value_t = String::from("Classicl Server"))]
    pub name: String,

    /// Server MOTD shown when connecting
    #[clap(short, long, value_parser, default_value_t = String::from("hosted with style"))]
    pub motd: String,

    #[clap(short, long, value_parser)]
    /// Player limit
    pub limit: Option<i8>,
}
