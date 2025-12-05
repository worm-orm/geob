use udled::{
    TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::coords::{CoordSegSegSegRef, MultiCoordSeqRef},
    util::read_u32,
};

#[derive(Clone, Copy)]
pub struct MultiPolygonRef<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl<'a> MultiPolygonRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(&self.bytes[1..], self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<MultiCoordSeqRef<'a>> {
        CoordSegSegSegRef::new(&self.bytes[1..], self.endian).get(idx)
    }
}

struct MultiPolygonType;

impl<'input> FromBytes<'input, &'input [u8]> for MultiPolygonType {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?;
        if ty.value != GeoType::MultiPolygon {
            return Err(reader.error("Expected a Multipolygon"))?;
        }

        Ok(Self)
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiPolygonRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(
            (
                MultiPolygonType::byteorder(byteorder),
                CoordSegSegSegRef::byteorder(byteorder),
            )
                .slice(),
        )?;

        Ok(Self {
            bytes: bytes.value,
            endian: byteorder,
        })
    }
}
