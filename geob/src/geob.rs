use crate::{
    GeoType, SRID,
    types::{GeobParser, GeobRef, GeometryRef, LineStringRef, PointRef, PolygonRef},
    util::{get_endian, read_u32, write_u32},
    wkt,
    writer::{BinaryWriter, ToBytes},
};
use alloc::{sync::Arc, vec::Vec};
use core::fmt;
use geo_traits::to_geo::ToGeoGeometry;

use udled::{Input, bytes::Endian};

#[derive(Clone)]
pub struct Geob(Arc<[u8]>);

impl fmt::Debug for Geob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GeomB").field(&self.geometry()).finish()
    }
}

impl fmt::Display for Geob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wkt::display_geometry(self, f)
    }
}

impl PartialEq for Geob {
    fn eq(&self, other: &Self) -> bool {
        self.srid() == other.srid() && self.geometry() == other.geometry()
    }
}

impl Geob {
    pub fn new_point(srid: u32, x: f64, y: f64) -> Result<Geob, <Vec<u8> as BinaryWriter>::Error> {
        let endian = Endian::native();
        let mut output = Vec::new();
        let bo = match endian {
            Endian::Big => 0u8,
            Endian::Lt => 1u8,
        };

        output.write_u8(bo)?;

        srid.write(&mut output, endian)?;

        GeoType::Point.write(&mut output, endian)?;

        x.write(&mut output, endian)?;
        y.write(&mut output, endian)?;

        let geob = Geob::new(output);

        Ok(geob)
    }

    pub fn from_text(input: &str) -> udled::Result<Geob> {
        wkt::parse(input, Endian::native())
    }

    pub fn from_bytes<T: Into<Vec<u8>> + AsRef<[u8]>>(bytes: T) -> Result<Geob, udled::Error> {
        let mut input = Input::new(bytes.as_ref());
        input.eat(GeobParser)?;

        let bytes: Vec<u8> = bytes.into();

        Ok(Self(Arc::from(bytes)))
    }

    pub fn srid(&self) -> u32 {
        read_u32(&self.0[1..], self.endian())
    }

    pub fn set_srid(&mut self, srid: SRID) {
        let endian = self.endian();
        write_u32(Arc::make_mut(&mut self.0), srid.into(), endian);
    }

    pub fn kind(&self) -> GeoType {
        GeoType::from_u8(self.0[5]).unwrap()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn endian(&self) -> Endian {
        get_endian(self.0[0]).unwrap()
    }

    pub fn geometry(&self) -> GeometryRef<'_> {
        GeobRef::new(&self.0).geometry()
    }

    pub fn as_point(&self) -> Option<PointRef<'_>> {
        match self.geometry() {
            GeometryRef::Point(line) => Some(line),
            _ => None,
        }
    }

    pub fn as_line_string(&self) -> Option<LineStringRef<'_>> {
        match self.geometry() {
            GeometryRef::LineString(line) => Some(line),
            _ => None,
        }
    }

    pub fn as_polygon(&self) -> Option<PolygonRef<'_>> {
        match self.geometry() {
            GeometryRef::Polygon(line) => Some(line),
            _ => None,
        }
    }
}

impl AsRef<[u8]> for Geob {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Geob {
    #[allow(unused)]
    pub(crate) fn slice_mut(&mut self) -> &mut [u8] {
        Arc::make_mut(&mut self.0)
    }

    pub(crate) fn slice(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn new(bytes: Vec<u8>) -> Geob {
        Geob(bytes.into())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Geob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.geometry().to_geometry().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Geob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let geo = geo_types::Geometry::deserialize(deserializer)?;
        Ok(Geob::from_geo_type(&geo, 0))
    }
}
