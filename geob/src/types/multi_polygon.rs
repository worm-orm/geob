use alloc::fmt;
use udled::bytes::{FromBytes, FromBytesExt};

use crate::types::coords::{CoordSegSegSegRef, MultiCoordSeqRef};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct MultiPolygonRef<'a>(CoordSegSegSegRef<'a>);

impl<'a> MultiPolygonRef<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, idx: usize) -> Option<MultiCoordSeqRef<'a>> {
        self.0.get(idx)
    }
}

impl<'a> fmt::Debug for MultiPolygonRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MultiPolygonRef")
            .field("lines", &self.0)
            .finish()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiPolygonRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(CoordSegSegSegRef::byteorder(byteorder))?;

        Ok(Self(bytes.value))
    }
}
