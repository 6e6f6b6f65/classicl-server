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

pub enum Command {
    Tp(String),
}

impl Command {
    pub fn from_str(s: &str) -> Result<Self, CommandError> {
        let split = s.split(' ').collect::<Vec<&str>>();

        if let Some(cmd) = split.first() {
            match *cmd {
                "tp" => {
                    if let Some(player) = split.get(1) {
                        if split.get(2).is_none() {
                            Ok(Self::Tp(player.to_string()))
                        } else {
                            Err(CommandError::TooManyArguments)
                        }
                    } else {
                        Err(CommandError::NotEnoughArguments)
                    }
                }
                _ => Err(CommandError::CommandNotKnown),
            }
        } else {
            Err(CommandError::NoCommand)
        }
    }
}

pub enum CommandError {
    NoCommand,
    CommandNotKnown,
    TooManyArguments,
    NotEnoughArguments,
}
