use alloc::fmt;
use udled::bytes::{Endian, FromBytes, FromBytesExt};

use crate::types::{
    CoordRef,
    coords::{CoordSeqIter, CoordSeqRef},
};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct MultiPointRef<'a>(CoordSeqRef<'a>);

impl<'a> MultiPointRef<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, idx: usize) -> Option<CoordRef<'a>> {
        self.0.get(idx)
    }

    pub fn iter(&self) -> CoordSeqIter<'a> {
        self.0.iter()
    }
}

impl<'a> fmt::Debug for MultiPointRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MultiPointRef")
            .field("points", &self.0)
            .finish()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiPointRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(CoordSeqRef::byteorder(byteorder))?;

        Ok(Self(bytes.value))
    }
}
