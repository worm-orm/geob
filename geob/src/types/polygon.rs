use udled::{
    TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::coords::{CoordSeqRef, MultiCoordSeqRef},
    util::read_u32,
};

#[derive(Clone, Copy)]
pub struct PolygonRef<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl<'a> PolygonRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(&self.bytes[1..], self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        MultiCoordSeqRef::new(&self.bytes[1..], self.endian).get(idx)
    }

    pub fn exterior(&self) -> Option<CoordSeqRef<'a>> {
        MultiCoordSeqRef::new(&self.bytes[1..], self.endian).get(0)
    }

    pub fn interior(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        MultiCoordSeqRef::new(&self.bytes[1..], self.endian).get(idx + 1)
    }
}

struct PolygonType;

impl<'input> FromBytes<'input, &'input [u8]> for PolygonType {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?;
        if ty.value != GeoType::Polygon {
            return Err(reader.error("Expected a polygon"))?;
        }

        Ok(Self)
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for PolygonRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(
            (
                PolygonType::byteorder(byteorder),
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
