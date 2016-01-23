use std::io::Read;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt};
use serde::de;
use serde::Deserialize;

use super::err::Error;

pub struct Deserializer<R : Read> {
    reader : R,
    expect_tag : bool,
}

impl<R> Deserializer<R> where R : Read {
    pub fn new(r : R) -> Self {
        Deserializer {
            reader : r,
            expect_tag : false,
        }
    }

    #[inline]
    pub fn uncompact_u32(&mut self) -> Result<u32, Error> {
        let mut b4 : [u8; 4] = [0; 4];
        b4[0] = try!(self.reader.read_u8());
        match b4[0] & 0xe0 {
            0xe0 => {
                self.reader.read_u32::<BigEndian>().map_err(From::from)
            }
            0xc0 => {
                b4[1] = try!(self.reader.read_u8());
                b4[2] = try!(self.reader.read_u8());
                b4[3] = try!(self.reader.read_u8());
                Ok(BigEndian::read_u32(&b4[0..3]) & ! 0xc0000000)
            }
            0xa0 | 0x80 => {
                b4[1] = try!(self.reader.read_u8());
                Ok((BigEndian::read_u16(&b4[0..1]) & ! 0x8000) as u32)
            }
            _ => {
                Ok(b4[0] as u32)
            }
        }
    }
}

impl<R> de::Deserializer for Deserializer<R> where R : Read {
    type Error = Error;


    /// This method walks a visitor through a value as it is being deserialized.
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        if self.expect_tag {
            let len = try!(self.uncompact_u32()) as usize;
            visitor.visit_usize(len)
        } else {
            unimplemented!();
        }
    }

    /// This method hints that the `Deserialize` type is expecting a `bool` value.
    #[inline]
    fn visit_bool<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u8());
        visitor.visit_bool(value != 0)
    }

    /// This method hints that the `Deserialize` type is expecting an `usize` value.
    #[inline]
    fn visit_usize<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u32::<BigEndian>());
        visitor.visit_usize(value as usize)
    }

    /// This method hints that the `Deserialize` type is expecting an `u8` value.
    #[inline]
    fn visit_u8<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u8());
        visitor.visit_u8(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `u16` value.
    #[inline]
    fn visit_u16<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u16::<BigEndian>());
        visitor.visit_u16(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `u32` value.
    #[inline]
    fn visit_u32<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u32::<BigEndian>());
        visitor.visit_u32(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `u64` value.
    #[inline]
    fn visit_u64<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u64::<BigEndian>());
        visitor.visit_u64(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `isize` value.
    #[inline]
    fn visit_isize<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_i32::<BigEndian>());
        visitor.visit_isize(value as isize)
    }

    /// This method hints that the `Deserialize` type is expecting an `i8` value.
    #[inline]
    fn visit_i8<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_i8());
        visitor.visit_i8(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `i16` value.
    #[inline]
    fn visit_i16<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_i16::<BigEndian>());
        visitor.visit_i16(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `i32` value.
    #[inline]
    fn visit_i32<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_i32::<BigEndian>());
        visitor.visit_i32(value)
    }

    /// This method hints that the `Deserialize` type is expecting an `i64` value.
    #[inline]
    fn visit_i64<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_i64::<BigEndian>());
        visitor.visit_i64(value)
    }

    /// This method hints that the `Deserialize` type is expecting a `f32` value.
    #[inline]
    fn visit_f32<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_f32::<BigEndian>());
        visitor.visit_f32(value)
    }

    /// This method hints that the `Deserialize` type is expecting a `f64` value.
    #[inline]
    fn visit_f64<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_f64::<BigEndian>());
        visitor.visit_f64(value)
    }

    /// This method hints that the `Deserialize` type is expecting a `char` value.
    #[inline]
    fn visit_char<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let value = try!(self.reader.read_u8());
        visitor.visit_char(value as char)
    }

    /// This method hints that the `Deserialize` type is expecting a `&str` value.
    #[inline]
    fn visit_str<V>(&mut self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        Err(<Error as de::Error>::syntax("unimplemented"))
    }

    /// This method hints that the `Deserialize` type is expecting a `String` value.
    #[inline]
    fn visit_string<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let len = try!(self.uncompact_u32()) as usize;
        let mut s = vec![0; len];
        try!(self.reader.read_exact(&mut s));
        visitor.visit_bytes(&s)
    }

    /// This method hints that the `Deserialize` type is expecting an `unit` value.
    #[inline]
    fn visit_unit<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_unit()
    }

    /// This method hints that the `Deserialize` type is expecting an `Option` value. This allows
    /// deserializers that encode an optional value as a nullable value to convert the null value
    /// into a `None`, and a regular value as `Some(value)`.
    #[inline]
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let len = try!(self.uncompact_u32());
        if len == 0 {
            visitor.visit_none()
        } else if len == 1 {
            visitor.visit_some(self)
        } else {
            Err(<Error as de::Error>::syntax("unimplemented"))
        }
    }

    /// This method hints that the `Deserialize` type is expecting a sequence value. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let len = try!(self.uncompact_u32());
        visitor.visit_seq(SeqVisitor::with_len(self, len as usize))
    }

    /// This method hints that the `Deserialize` type is expecting a map of values. This allows
    /// deserializers to parse sequences that aren't tagged as maps.
    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let len = try!(self.uncompact_u32());
        visitor.visit_map(MapVisitor::with_len(self, len as usize))
    }

    /// This method hints that the `Deserialize` type is expecting a unit struct. This allows
    /// deserializers to a unit struct that aren't tagged as a unit struct.
    #[inline]
    fn visit_unit_struct<V>(&mut self,
                            _name: &'static str,
                            visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.visit_unit(visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a newtype struct. This allows
    /// deserializers to a newtype struct that aren't tagged as a newtype struct.
    #[inline]
    fn visit_newtype_struct<V>(&mut self,
                               name: &'static str,
                               visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.visit_tuple_struct(name, 1, visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a tuple struct. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    #[inline]
    fn visit_tuple_struct<V>(&mut self,
                             _name: &'static str,
                             len: usize,
                             visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.visit_tuple(len, visitor)
    }

    /// This method hints that the `Deserialize` type is expecting a struct. This allows
    /// deserializers to parse sequences that aren't tagged as maps.
    #[inline]
    fn visit_struct<V>(&mut self,
                       _name: &'static str,
                       fields: &'static [&'static str],
                       mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(SeqVisitor::with_len(self, fields.len()))
    }

    /// This method hints that the `Deserialize` type is expecting a tuple value. This allows
    /// deserializers that provide a custom tuple serialization to properly deserialize the type.
    #[inline]
    fn visit_tuple<V>(&mut self, len: usize, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(SeqVisitor::with_len(self, len as usize))
    }

    /// This method hints that the `Deserialize` type is expecting an enum value. This allows
    /// deserializers that provide a custom enumeration serialization to properly deserialize the
    /// type.
    #[inline]
    fn visit_enum<V>(&mut self,
                     _enum: &'static str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::EnumVisitor,
    {
        visitor.visit(self)
        //TODO VariantVisitor
    }

    /// This method hints that the `Deserialize` type is expecting a `Vec<u8>`. This allows
    /// deserializers that provide a custom byte vector serialization to properly deserialize the
    /// type.
    #[inline]
    fn visit_bytes<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        let len = try!(self.uncompact_u32()) as usize;
        let mut s = vec![0; len];
        try!(self.reader.read_exact(&mut s));
        visitor.visit_bytes(&s)
    }

    /// Specify a format string for the deserializer.
    ///
    /// The deserializer format is used to determine which format
    /// specific field attributes should be used with the
    /// deserializer.
    fn format() -> &'static str {
        "pwrd"
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct SeqVisitor<'a, R : Read + 'a> {
    de : &'a mut Deserializer<R>,
    len : usize,
    cur : usize,
}

impl<'a, R> SeqVisitor<'a, R> where R : Read + 'a {
    fn with_len(de : &'a mut Deserializer<R>, len : usize) -> Self {
        SeqVisitor {
            de : de,
            len : len,
            cur : 0,
        }
    }
}

/// `SeqVisitor` visits each item in a sequence.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation, which deserializes
/// each item in a sequence.
impl<'a, R> de::SeqVisitor for SeqVisitor<'a, R> where R : Read + 'a {
    /// The error type that can be returned if some error occurs during deserialization.
    type Error = Error;

    /// This returns a `Ok(Some(value))` for the next value in the sequence, or `Ok(None)` if there
    /// are no more remaining items.
    fn visit<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: Deserialize
    {
        if self.cur >= self.len {
            Ok(None)
        } else {
            self.cur += 1;
            let t = try!(T::deserialize(self.de));
            Ok(Some(t))
        }
    }

    /// This signals to the `SeqVisitor` that the `Visitor` does not expect any more items.
    fn end(&mut self) -> Result<(), Self::Error> {
        if self.cur >= self.len {
            Ok(())
        } else {
            Err(<Error as de::Error>::syntax("impossible"))
        }
    }

    /// Return the lower and upper bound of items remaining in the sequence.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct MapVisitor<'a, R : Read + 'a> {
    de : &'a mut Deserializer<R>,
    len : usize,
    cur : usize,
}

impl<'a, R> MapVisitor<'a, R> where R : Read + 'a {
    fn with_len(de : &'a mut Deserializer<R>, len : usize) -> Self {
        MapVisitor {
            de : de,
            len : len,
            cur : 0,
        }
    }
}

/// `MapVisitor` visits each item in a sequence.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation.
impl<'a, R> de::MapVisitor for MapVisitor<'a, R> where R : Read + 'a {
    /// The error type that can be returned if some error occurs during deserialization.
    type Error = Error;

    /// This returns a `Ok(Some((key, value)))` for the next (key-value) pair in the map, or
    /// `Ok(None)` if there are no more remaining items.
    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        if self.cur >= self.len {
            Ok(None)
        } else {
            self.cur += 1;
            let key = try!(K::deserialize(self.de));
            let value = try!(V::deserialize(self.de));
            Ok(Some((key, value)))
        }
    }

    /// This returns a `Ok(Some(key))` for the next key in the map, or `Ok(None)` if there are no
    /// more remaining items.
    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: Deserialize
    {
        let key = try!(K::deserialize(self.de));
        Ok(Some(key))
    }

    /// This returns a `Ok(value)` for the next value in the map.
    #[inline]
    fn visit_value<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize
    {
        let value = try!(V::deserialize(self.de));
        Ok(value)
    }

    /// This signals to the `MapVisitor` that the `Visitor` does not expect any more items.
    #[inline]
    fn end(&mut self) -> Result<(), Self::Error> {
        if self.cur >= self.len {
            Ok(())
        } else {
            Err(<Error as de::Error>::syntax("impossible"))
        }
    }

    /// Return the lower and upper bound of items remaining in the sequence.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    /// Report that there 
    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Self::Error>
        where V: Deserialize,
    {
        Err(<Error as de::Error>::missing_field(field))
    }
}

///////////////////////////////////////////////////////////////////////////////
/*
/// `EnumVisitor` is a visitor that is created by the `Deserialize` and passed to the
/// `Deserializer` in order to deserialize enums.
pub trait EnumVisitor {
    /// The value produced by this visitor.
    type Value;

    /// Visit the specific variant with the `VariantVisitor`.
    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: VariantVisitor;
}
*/
///////////////////////////////////////////////////////////////////////////////

/// `VariantVisitor` is a visitor that is created by the `Deserializer` and passed to the
/// `Deserialize` in order to deserialize a specific enum variant.
impl<R> de::VariantVisitor for Deserializer<R> where R : Read {
    /// The error type that can be returned if some error occurs during deserialization.
    type Error = Error;

    /// `visit_variant` is called to identify which variant to deserialize.
    #[inline]
    fn visit_variant<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize
    {
        self.expect_tag = true;
        let val = try!(de::Deserialize::deserialize(self));
        self.expect_tag = false;
        Ok(val)
        //TODO
    }

    /// `visit_unit` is called when deserializing a variant with no values.
    #[inline]
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// `visit_newtype` is called when deserializing a variant with a single value. By default this
    /// uses the `visit_tuple` method to deserialize the value.
    #[inline]
//    fn visit_newtype<T>(&mut self) -> Result<T, Self::Error>
//        where T: Deserialize,

    /// `visit_tuple` is called when deserializing a tuple-like variant.
    fn visit_tuple<V>(&mut self,
                      len: usize,
                      mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_seq(SeqVisitor::with_len(self, len))
    }

    /// `visit_struct` is called when deserializing a struct-like variant.
    fn visit_struct<V>(&mut self,
                       fields: &'static [&'static str],
                       mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        visitor.visit_seq(SeqVisitor::with_len(self, fields.len()))
    }
}
