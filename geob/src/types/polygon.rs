use crate::types::coords::{CoordSeqRef, MultiCoordSeqRef};
use alloc::fmt;
use udled::bytes::{FromBytes, FromBytesExt};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct PolygonRef<'a>(pub(crate) MultiCoordSeqRef<'a>);

impl<'a> PolygonRef<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        self.0.get(idx)
    }

    pub fn exterior(&self) -> Option<CoordSeqRef<'a>> {
        self.0.get(0)
    }

    pub fn interior(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        self.0.get(1 + idx)
    }
}

impl<'a> fmt::Debug for PolygonRef<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PolygonRef")
            .field("rings", &self.0)
            .finish()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for PolygonRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(MultiCoordSeqRef::byteorder(byteorder))?;

        Ok(Self(bytes.value))
    }
}
