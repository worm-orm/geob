use alloc::fmt;
use udled::{
    TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
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
        read_u32(self.bytes, self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        MultiCoordSeqRef::new(self.bytes, self.endian).get(idx)
    }

    pub fn iter(&self) -> MultiCoordSeqIter<'a> {
        MultiCoordSeqRef::new(self.bytes, self.endian).iter()
    }
}

impl<'a> fmt::Debug for MultiLineStringRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MultiLineStringRef")
            .field("lines", &MultiCoordSeqRef::new(self.bytes, self.endian))
            .finish()
    }
}

impl<'a> PartialEq for MultiLineStringRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        MultiCoordSeqRef::new(self.bytes, self.endian)
            == MultiCoordSeqRef::new(&other.bytes, other.endian)
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiLineStringRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(MultiCoordSeqRef::byteorder(byteorder).slice())?;

        Ok(Self {
            bytes: bytes.value,
            endian: byteorder,
        })
    }
}
