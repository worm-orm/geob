mod collection;
mod coords;
mod geometry;
mod line_string;
mod muli_line_string;
mod multi_point;
mod multi_polygon;
mod point;
mod polygon;
mod types;

use alloc::fmt;
use udled::{
    AsSlice, Input, Tokenizer,
    bytes::{Endian, FromBytesExt},
};

use crate::{
    Geob,
    util::{get_endian, read_u32},
    wkt,
};

pub use self::{
    collection::CollectionRef,
    coords::CoordRef,
    coords::{CoordSeqRef, MultiCoordSeqRef},
    geometry::GeometryRef,
    line_string::LineStringRef,
    muli_line_string::*,
    multi_point::*,
    multi_polygon::MultiPolygonRef,
    point::{Point, PointRef},
    polygon::PolygonRef,
    types::*,
};

#[derive(Clone, Copy)]
pub struct GeobRef<'a> {
    pub bytes: &'a [u8],
}

impl<'a> GeobRef<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> GeobRef<'a> {
        GeobRef { bytes }
    }

    pub fn to_owned(&self) -> Geob {
        unsafe { Geob::from_bytes_unchecked(self.bytes) }
    }

    pub fn from_bytes(bytes: &'a [u8]) -> udled::Result<GeobRef<'a>> {
        let mut input = Input::new(bytes.as_ref());
        input.parse(GeobParser)
    }
}

impl<'a> GeobRef<'a> {
    pub fn geometry(&self) -> GeometryRef<'a> {
        let endian = get_endian(self.bytes[0]).unwrap();
        Input::new(&self.bytes[5..])
            .parse(GeometryRef::byteorder(endian))
            .map(|m| m.value)
            .unwrap()
    }

    pub fn srid(&self) -> u32 {
        read_u32(&self.bytes[1..], self.endian())
    }

    pub fn endian(&self) -> Endian {
        get_endian(self.bytes[0]).unwrap()
    }
}

impl<'a> fmt::Debug for GeobRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GeomB").field(&self.geometry()).finish()
    }
}

impl<'a> fmt::Display for GeobRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wkt::display_geometry(*self, f)
    }
}

impl<'a> PartialEq for GeobRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.srid() == other.srid() && self.geometry() == other.geometry()
    }
}

pub struct GeobParser;

impl<'input> Tokenizer<'input, &'input [u8]> for GeobParser {
    type Token = GeobRef<'input>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
    ) -> Result<Self::Token, udled::Error> {
        let start = reader.position();
        let endian = get_endian(reader.read()?)
            .ok_or_else(|| reader.error("Expected 0 (big) or 1 (little) endian"))?;

        reader.eat(u32::byteorder(endian))?;

        let geo = reader.parse(GeometryRef::byteorder(endian))?;

        let span = geo.span.with_start(start);

        let bytes = reader
            .buffer()
            .sliced(span)
            .ok_or_else(|| reader.error("Expected slice"))?;

        Ok(GeobRef { bytes })
    }
}
