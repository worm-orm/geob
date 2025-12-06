use alloc::fmt;
use udled::bytes::{Endian, FromBytes, FromBytesExt};

use crate::types::{
    CoordRef,
    coords::{CoordSeqIter, CoordSeqRef},
};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct LineStringRef<'a>(pub(crate) CoordSeqRef<'a>);

impl<'a> fmt::Debug for LineStringRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LineStringRef")
            .field("coords", &self.0)
            .finish()
    }
}

impl<'a> LineStringRef<'a> {
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

impl<'input> FromBytes<'input, &'input [u8]> for LineStringRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(CoordSeqRef::byteorder(byteorder))?;
        Ok(Self(bytes.value))
    }
}
