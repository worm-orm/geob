use alloc::{borrow::Cow, fmt};

use crate::writer::ToBytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Unit {
    Meter,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum CRS {
    Geodetic,
    Projected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum CoordinateSystem {
    Spherical,
    Ellipsoidal,
    Cart2d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Datum {
    WGS84,
    ETRS89,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EPSG {
    id: u32,
    name: Cow<'static, str>,
    unit: Unit,
    datum: Datum,
    crs: CRS,
    cs: CoordinateSystem,
}

impl EPSG {
    pub const WEB_MERCATOR: EPSG = EPSG {
        id: 3857,
        name: Cow::Borrowed("Web Mercator"),
        unit: Unit::Meter,
        datum: Datum::WGS84,
        crs: CRS::Projected,
        cs: CoordinateSystem::Cart2d,
    };

    pub const WGS84: EPSG = EPSG {
        id: 4326,
        name: Cow::Borrowed("WGS84"),
        unit: Unit::Meter,
        datum: Datum::WGS84,
        crs: CRS::Projected,
        cs: CoordinateSystem::Ellipsoidal,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SRID(u32);

impl From<SRID> for u32 {
    fn from(value: SRID) -> Self {
        value.0
    }
}

impl From<u32> for SRID {
    fn from(value: u32) -> Self {
        SRID(value)
    }
}

impl SRID {
    pub const WEB_MERCATOR: SRID = SRID(EPSG::WEB_MERCATOR.id);
    pub const WGS84: SRID = SRID(EPSG::WGS84.id);
}

impl fmt::Display for SRID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToBytes for SRID {
    fn write<W: crate::writer::BinaryWriter>(
        &self,
        output: &mut W,
        endian: udled::bytes::Endian,
    ) -> Result<(), W::Error> {
        self.0.write(output, endian)
    }
}
