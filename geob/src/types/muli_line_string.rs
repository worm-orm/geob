use udled::{
    TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::coords::{CoordSeqRef, MultiCoordSeqIter, MultiCoordSeqRef},
    util::read_u32,
};

#[derive(Clone, Copy)]
pub struct MultiLineStringRef<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl<'a> MultiLineStringRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(&self.bytes[1..], self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        MultiCoordSeqRef::new(&self.bytes[1..], self.endian).get(idx)
    }

    pub fn iter(&self) -> MultiCoordSeqIter<'a> {
        MultiCoordSeqRef::new(&self.bytes[1..], self.endian).iter()
    }
}

struct MultiLineStringType;

impl<'input> FromBytes<'input, &'input [u8]> for MultiLineStringType {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?;
        if ty.value != GeoType::MultiLineString {
            return Err(reader.error("Expected a multilinestring"))?;
        }

        Ok(Self)
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiLineStringRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(
            (
                MultiLineStringType::byteorder(byteorder),
                MultiCoordSeqRef::byteorder(byteorder),
            )
                .slice(),
        )?;

        Ok(Self {
            bytes: bytes.value,
            endian: byteorder,
        })
    }
}
