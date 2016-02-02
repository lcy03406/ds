use std::io;
use byteorder::{BigEndian, WriteBytesExt};
use serde;
use serde::ser;
use serde::Serialize;

use serde::Serializer as SerdeSerializer;

use super::err::Error;

pub struct Serializer<W : io::Write> {
    writer : W
}

impl<W> Serializer<W> where W : io::Write {
    pub fn new(w : W) -> Self {
        Serializer {
            writer : w
        }
    }

    #[inline]
    pub fn compact_u32(&mut self, value: u32) -> Result<(), Error> {
        if value < 0x80 {
            self.writer.write_u8(value as u8).map_err(From::from)
        } else if value < 0x4000 {
            self.writer.write_u16::<BigEndian>((value|0x8000) as u16).map_err(From::from)
        } else if value < 0x20000000 {
            self.writer.write_u32::<BigEndian>((value|0xc0000000) as u32).map_err(From::from)
        } else {
            try!(self.writer.write_u8(0xe0));//.map_err(From::from));
            self.writer.write_u32::<BigEndian>(value).map_err(From::from)
        }
    }
}

#[inline]
fn parse_tag(name : &'static str) -> usize {
    const PROTOCOL_TAG : &'static str = "ProtocolFrom";
    if name.starts_with(PROTOCOL_TAG) {
        name[PROTOCOL_TAG.len()..].parse().unwrap()
    } else {
        0
    }
}

impl<W> serde::Serializer for Serializer<W> where W : io::Write {
    type Error = Error;

    #[inline]
    fn visit_bool(&mut self, value: bool) -> Result<(), Self::Error> {
        self.visit_u8(value as u8)
    }

    #[inline]
    fn visit_isize(&mut self, value: isize) -> Result<(), Self::Error> {
        self.visit_i32(value as i32)
    }

    #[inline]
    fn visit_i8(&mut self, value: i8) -> Result<(), Self::Error> {
        self.writer.write_i8(value).map_err(From::from)
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> Result<(), Self::Error> {
        self.writer.write_i16::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> Result<(), Self::Error> {
        self.writer.write_i32::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> Result<(), Self::Error> {
        self.writer.write_i64::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        self.visit_u32(value as u32)
    }

    #[inline]
    fn visit_u8(&mut self, value: u8) -> Result<(), Self::Error> {
        self.writer.write_u8(value).map_err(From::from)
    }

    #[inline]
    fn visit_u16(&mut self, value: u16) -> Result<(), Self::Error> {
        self.writer.write_u16::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_u32(&mut self, value: u32) -> Result<(), Self::Error> {
        self.writer.write_u32::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> Result<(), Self::Error> {
        self.writer.write_u64::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_f32(&mut self, value: f32) -> Result<(), Self::Error> {
        self.writer.write_f32::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> Result<(), Self::Error> {
        self.writer.write_f64::<BigEndian>(value).map_err(From::from)
    }

    #[inline]
    fn visit_char(&mut self, value: char) -> Result<(), Self::Error> {
        self.visit_u8(value as u8)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> Result<(), Self::Error> {
        try!(self.compact_u32(value.len() as u32));
        self.writer.write_all(value.as_bytes()).map_err(From::from)
    }

    #[inline]
    fn visit_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        try!(self.compact_u32(value.len() as u32));
        self.writer.write_all(value).map_err(From::from)
    }

    #[inline]
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn visit_unit_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
        self.visit_unit()
    }

    #[inline]
    fn visit_unit_variant(&mut self,
                          name: &'static str,
                          variant_index: usize,
                          _variant: &'static str) -> Result<(), Self::Error> {
        let tag_offset = parse_tag(name);
        self.compact_u32((tag_offset + variant_index) as u32)
    }

    /// Serializes Option<T> as Vec<T> of length 0 or 1.
    #[inline]
    fn visit_none(&mut self) -> Result<(), Self::Error> {
        self.compact_u32(0)
    }

    /// Serializes Option<T> as Vec<T> of length 0 or 1.
    #[inline]
    fn visit_some<V>(&mut self, value: V) -> Result<(), Self::Error>
        where V: Serialize
    {
        try!(self.compact_u32(1));
        value.serialize(self)
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: ser::SeqVisitor,
    {
        match visitor.len() {
            Some(len) => {
                try!(self.compact_u32(len as u32));
                while let Some(()) = try!(visitor.visit(self)) { }
                Ok(())
            }
            None => {
                unimplemented!();
            }
        }
    }

    #[inline]
    fn visit_seq_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize,
    {
        value.serialize(self)
    }

    /// Serializes Tuple as Struct , does not serialize length
    #[inline]
    fn visit_tuple<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: ser::SeqVisitor,
    {
        while let Some(()) = try!(visitor.visit(self)) { }
        Ok(())
    }

    #[inline]
    fn visit_tuple_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn visit_tuple_struct<V>(&mut self,
                             _name: &'static str,
                             visitor: V) -> Result<(), Self::Error>
        where V: ser::SeqVisitor,
    {
        self.visit_tuple(visitor)
    }

    #[inline]
    fn visit_tuple_struct_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        self.visit_tuple_elt(value)
    }

    #[inline]
    fn visit_tuple_variant<V>(&mut self,
                              name: &'static str,
                              variant_index: usize,
                              variant: &'static str,
                              visitor: V) -> Result<(), Self::Error>
        where V: ser::SeqVisitor,
    {
        let tag_offset = parse_tag(name);
        try!(self.compact_u32((tag_offset + variant_index) as u32));
        if tag_offset > 0 {
            let mut inner = Serializer::new(Vec::new());
            try!(inner.visit_tuple_struct(variant, visitor));
            self.visit_bytes(&*inner.writer)
        } else {
            self.visit_tuple_struct(variant, visitor)
        }
    }

    #[inline]
    fn visit_tuple_variant_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize
    {
        self.visit_tuple_struct_elt(value)
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<(), Self::Error>
        where V: ser::MapVisitor,
    {
        match visitor.len() {
            Some(len) => {
                try!(self.compact_u32(len as u32));
                while let Some(()) = try!(visitor.visit(self)) { }
                Ok(())
            }
            None => {
                unimplemented!();
            }
        }
    }

    #[inline]
    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
        where K: Serialize,
              V: Serialize,
    {
        try!(key.serialize(self));
        value.serialize(self)
    }

    #[inline]
    fn visit_struct<V>(&mut self, _name: &'static str, mut visitor: V) -> Result<(), Self::Error>
        where V: ser::MapVisitor,
    {
        while let Some(()) = try!(visitor.visit(self)) { }
        Ok(())
    }

    #[inline]
    fn visit_struct_elt<V>(&mut self, _key: &'static str, value: V) -> Result<(), Self::Error>
        where V: Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn visit_struct_variant<V>(&mut self,
                               name: &'static str,
                               variant_index: usize,
                               variant: &'static str,
                               visitor: V) -> Result<(), Self::Error>
        where V: ser::MapVisitor,
    {
        let tag_offset = parse_tag(name);
        try!(self.compact_u32((tag_offset + variant_index) as u32));
        if tag_offset > 0 {
            let mut inner = Serializer::new(Vec::new());
            try!(inner.visit_struct(variant, visitor));
            self.visit_bytes(&*inner.writer)
        } else {
            self.visit_struct(variant, visitor)
        }
    }

    #[inline]
    fn visit_struct_variant_elt<V>(&mut self,
                                   key: &'static str,
                                   value: V) -> Result<(), Self::Error>
        where V: Serialize,
    {
        self.visit_struct_elt(key, value)
    }

    #[inline]
    fn format() -> &'static str {
        "pwrd"
    }
}
