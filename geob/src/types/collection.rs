use alloc::fmt;
use udled::{
    AsSlice, Input, TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::{geometry::GeometryRef, types::TYPE_LEN},
    util::read_u32,
};

#[derive(Clone, Copy)]
pub struct CollectionRef<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl<'a> CollectionRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(&self.bytes[TYPE_LEN..], self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<GeometryRef<'a>> {
        if idx >= self.len() {
            return None;
        }

        let mut input = Input::new(self.bytes);

        if idx == 0 {
            input
                .parse(GeometryRef::byteorder(self.endian))
                .map(|m| m.value)
                .ok()
        } else {
            for i in 0..=idx {
                if i == idx {
                    return input
                        .parse(GeometryRef::byteorder(self.endian))
                        .map(|m| m.value)
                        .ok();
                } else {
                    input.parse(GeometryRef::byteorder(self.endian)).ok();
                }
            }

            None
        }
    }
}

impl<'a> fmt::Debug for CollectionRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl<'a> PartialEq for CollectionRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

struct CollectionType;

impl<'input> FromBytes<'input, &'input [u8]> for CollectionType {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?;
        if ty.value != GeoType::Collection {
            return Err(reader.error("Expected a collection"))?;
        }

        Ok(Self)
    }
}

impl<'a> FromBytes<'a, &'a [u8]> for CollectionRef<'a> {
    fn parse(
        reader: &mut udled::Reader<'_, 'a, &'a [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let start = reader.parse(CollectionType::byteorder(byteorder).spanned())?;
        let len = reader.parse(u32::byteorder(byteorder))?;
        let coords = reader.parse(
            GeometryRef::byteorder(byteorder)
                .repeat(len.value as _)
                .spanned(),
        )?;

        let span = start + coords;

        Ok(CollectionRef {
            bytes: reader.buffer().sliced(span).unwrap(),
            endian: byteorder,
        })
    }
}
