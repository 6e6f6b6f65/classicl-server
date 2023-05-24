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

pub use classicl_derive::FixedSize;
use de::Deserializer;
use error::Result;
pub use length::FixedSize;
use ser::Serializer;
use serde::{Deserialize, Serialize};

mod error;

mod de;
mod length;
mod ser;

pub fn to_bytes<T>(value: T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer { output: vec![] };

    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn from_bytes<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(b);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(error::Error::WrongPacket)
    }
}
