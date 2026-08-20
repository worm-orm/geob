use crate::types::coords::{CoordSeqRef, MultiCoordSeq, MultiCoordSeqRef};
use alloc::fmt;
use udled::bytes::{FromBytes, FromBytesExt};

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Polygon(MultiCoordSeq);

impl Polygon {
    pub fn new(rings: MultiCoordSeq) -> Self {
        Self(rings)
    }
}

impl core::ops::Deref for Polygon {
    type Target = MultiCoordSeq;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PolygonRef<'_>> for Polygon {
    fn from(polygon_ref: PolygonRef<'_>) -> Self {
        Self(MultiCoordSeq::from(polygon_ref.0))
    }
}

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

    pub fn iter(&self) -> impl Iterator<Item = CoordSeqRef<'a>> + '_ {
        self.0.iter()
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
