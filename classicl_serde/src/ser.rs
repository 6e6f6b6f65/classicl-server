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

use crate::error::{Error, Result};
use serde::{ser, Serialize};
pub(crate) struct Serializer {
    pub output: Vec<u8>,
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.push(v.to_be_bytes()[0]);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.append(&mut Vec::from(v.to_be_bytes()));
        Ok(())
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.push(v.to_be_bytes()[0]);
        Ok(())
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let v = v.to_string();
        let mut v: Vec<u8> = v.as_bytes().to_vec();
        v.resize(64, 0x20);
        self.output.append(&mut v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let mut v: Vec<u8> = Vec::from(v);
        v.resize(1024, 0);
        self.output.append(&mut v);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::NotSupported)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::NotSupported)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::NotSupported)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::NotSupported)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::NotSupported)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::NotSupported)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn end(self) -> Result<()> {
        Err(Error::NotSupported)
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn end(self) -> Result<()> {
        Err(Error::NotSupported)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn end(self) -> Result<()> {
        Err(Error::NotSupported)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn end(self) -> Result<()> {
        Err(Error::NotSupported)
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::NotSupported)
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::NotSupported)
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::NotSupported)
    }
}
