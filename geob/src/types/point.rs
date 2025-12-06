use alloc::fmt;
use udled::bytes::{Endian, FromBytes, FromBytesExt};

use crate::{
    GeoType,
    types::coords::CoordRef,
    util::{get_endian, read_f64, write_f64},
    writer::ToBytes,
};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct PointRef<'a>(pub(crate) CoordRef<'a>);

impl<'a> PointRef<'a> {
    pub fn coord(&self) -> CoordRef<'a> {
        self.0
    }

    pub fn x(&self) -> f64 {
        self.0.x()
    }

    pub fn y(&self) -> f64 {
        self.0.y()
    }
}

impl<'a> fmt::Debug for PointRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PointRef")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for PointRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(CoordRef::byteorder(byteorder))?;

        Ok(Self(bytes.value))
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Point {
    bytes: [u8; 18],
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() && self.y() == other.y()
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        let mut bytes: [u8; 18] = [0; 18];
        let endian = Endian::native();
        match endian {
            Endian::Big => {
                bytes[0] = 0;
            }
            Endian::Lt => {
                bytes[0] = 1;
            }
        }

        bytes[1] = GeoType::Point as _;

        write_f64(&mut bytes[2..], x, endian);
        write_f64(&mut bytes[10..], y, endian);

        Point { bytes }
    }

    fn endian(&self) -> Endian {
        get_endian(self.bytes[0]).unwrap()
    }

    pub fn x(&self) -> f64 {
        read_f64(&self.bytes[2..], self.endian())
    }

    pub fn y(&self) -> f64 {
        read_f64(&self.bytes[10..], self.endian())
    }
}

impl ToBytes for Point {
    fn write<W: crate::writer::BinaryWriter>(
        &self,
        output: &mut W,
        endian: Endian,
    ) -> Result<(), W::Error> {
        GeoType::Point.write(output, endian)?;
        self.x().write(output, endian)?;
        self.y().write(output, endian)?;
        Ok(())
    }
}
