use core::fmt::write;

use udled::{
    TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::coords::CoordRef,
    util::{get_endian, read_f64, write_f64},
    writer::{BinaryWriter, ToBytes},
};

#[derive(Clone, Copy)]
pub struct PointRef<'a> {
    bytes: &'a [u8],
    endian: Endian,
}

impl<'a> PointRef<'a> {
    pub fn coord(&self) -> CoordRef<'a> {
        CoordRef::new(&self.bytes[1..], self.endian)
    }

    pub fn x(&self) -> f64 {
        CoordRef::new(&self.bytes[1..], self.endian).x()
    }

    pub fn y(&self) -> f64 {
        CoordRef::new(&self.bytes[1..], self.endian).y()
    }
}

struct PointType;

impl<'input> FromBytes<'input, &'input [u8]> for PointType {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?;
        if ty.value != GeoType::Point {
            return Err(reader.error("Expected a point"))?;
        }

        Ok(Self)
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for PointRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let bytes = reader.parse(
            (
                PointType::byteorder(byteorder),
                CoordRef::byteorder(byteorder),
            )
                .slice(),
        )?;

        Ok(Self {
            bytes: bytes.value,
            endian: byteorder,
        })
    }

    fn is(reader: &mut udled::Reader<'_, 'input, &'input [u8]>, byteorder: Endian) -> bool {
        reader.is(PointType::byteorder(byteorder))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct Point {
    bytes: [u8; 18],
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() && self.y() == other.y()
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
