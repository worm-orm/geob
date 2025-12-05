use udled::{
    TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::{
        CoordRef,
        coords::{CoordSeqIter, CoordSeqRef},
        types::TYPE_LEN,
    },
    util::read_u32,
};

#[derive(Clone, Copy)]
pub struct MultiPointRef<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl<'a> MultiPointRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(&self.bytes[1..], self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<CoordRef<'a>> {
        CoordSeqRef::new(&self.bytes[TYPE_LEN..], self.endian).get(idx)
    }

    pub fn iter(&self) -> CoordSeqIter<'a> {
        CoordSeqRef::new(&self.bytes[TYPE_LEN..], self.endian).iter()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiPointRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(
            (
                MultiPointType::byteorder(byteorder),
                CoordSeqRef::byteorder(byteorder),
            )
                .slice(),
        )?;

        Ok(Self {
            bytes: bytes.value,
            endian: byteorder,
        })
    }
}

struct MultiPointType;

impl<'input> FromBytes<'input, &'input [u8]> for MultiPointType {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?;
        if ty.value != GeoType::MultiPoint {
            return Err(reader.error("Expected a linestring"))?;
        }

        Ok(Self)
    }
}
