use core::{fmt, mem::transmute};

use udled::{
    AsBytes, AsSlice, Buffer,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::writer::ToBytes;

pub const TYPE_LEN: usize = 1;

pub const ENDIAN_LEN: usize = 1;

pub const SRID_LEN: usize = size_of::<u32>();

pub const GEOB_HEADER: usize = ENDIAN_LEN + SRID_LEN;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum GeoType {
    Point = 1,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    Collection,
}

impl fmt::Display for GeoType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            GeoType::Point => "POINT",
            GeoType::LineString => "LINESTRING",
            GeoType::Polygon => "POLYGON",
            GeoType::MultiPoint => "MULTIPOINT",
            GeoType::MultiLineString => "MULTILINESTRING",
            GeoType::MultiPolygon => "MULTIPOLYGON",
            GeoType::Collection => "GEOMETRYCOLLECTION",
        };
        f.write_str(s)
    }
}

impl GeoType {
    pub fn from_u8(i: u8) -> Option<GeoType> {
        if i > 0 && i <= 7 {
            let ty = unsafe { transmute::<u8, GeoType>(i) };
            Some(ty)
        } else {
            None
        }
    }
}

impl<'input, B> FromBytes<'input, B> for GeoType
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let ty = reader.parse(u8::byteorder(byteorder))?;
        if ty.value > 0 && ty.value <= 7 {
            let ty = unsafe { transmute::<u8, GeoType>(ty.value) };
            Ok(ty)
        } else {
            Err(reader.error("GeoType"))
        }
    }
}

impl ToBytes for GeoType {
    fn write<W: crate::writer::BinaryWriter>(
        &self,
        output: &mut W,
        _endian: Endian,
    ) -> Result<(), W::Error> {
        output.write_u8(*self as _)
    }
}
