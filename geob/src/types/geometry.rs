use udled::{
    Input,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::{
    GeoType,
    types::{
        LineStringRef, MultiLineStringRef, MultiPointRef, MultiPolygonRef, Point, PointRef,
        PolygonRef, collection::CollectionRef,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GeometryRef<'a> {
    Point(PointRef<'a>),
    LineString(LineStringRef<'a>),
    MultiPoint(MultiPointRef<'a>),
    MultiLineString(MultiLineStringRef<'a>),
    Polygon(PolygonRef<'a>),
    MultiPolygon(MultiPolygonRef<'a>),
    Collection(CollectionRef<'a>),
}

impl<'a> GeometryRef<'a> {
    pub fn validate(bytes: &[u8], endian: Endian) -> Result<(), udled::Error> {
        Input::new(bytes).eat(GeometryRef::byteorder(endian))
    }

    pub fn is(&self, ty: GeoType) -> bool {
        match (self, ty) {
            (GeometryRef::Point(_), GeoType::Point) => true,
            (GeometryRef::LineString(_), GeoType::LineString) => true,
            (GeometryRef::MultiPoint(_), GeoType::MultiPoint) => true,
            (GeometryRef::MultiLineString(_), GeoType::MultiLineString) => true,
            (GeometryRef::Polygon(_), GeoType::Polygon) => true,
            (GeometryRef::MultiPolygon(_), GeoType::MultiPolygon) => true,
            (GeometryRef::Collection(_), GeoType::Collection) => true,
            _ => false,
        }
    }
}

impl<'a> FromBytes<'a, &'a [u8]> for GeometryRef<'a> {
    fn parse(
        reader: &mut udled::Reader<'_, 'a, &'a [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let ty = reader.parse(GeoType::byteorder(byteorder))?.value;

        let geo = match ty {
            GeoType::Point => GeometryRef::Point(
                reader
                    .parse(PointRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
            GeoType::LineString => GeometryRef::LineString(
                reader
                    .parse(LineStringRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
            GeoType::Polygon => GeometryRef::Polygon(
                reader
                    .parse(PolygonRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
            GeoType::MultiPoint => GeometryRef::MultiPoint(
                reader
                    .parse(MultiPointRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
            GeoType::MultiLineString => GeometryRef::MultiLineString(
                reader
                    .parse(MultiLineStringRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
            GeoType::MultiPolygon => GeometryRef::MultiPolygon(
                reader
                    .parse(MultiPolygonRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
            GeoType::Collection => GeometryRef::Collection(
                reader
                    .parse(CollectionRef::byteorder(byteorder))
                    .map(|i| i.value)?,
            ),
        };

        Ok(geo)
    }
}

#[derive(Clone)]
pub enum Geometry {
    Point(Point),
}
