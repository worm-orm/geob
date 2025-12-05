use udled::bytes::{FromBytes, FromBytesExt};

use crate::{
    GeoType,
    types::{
        LineStringRef, MultiLineStringRef, MultiPointRef, MultiPolygonRef, Point, PointRef,
        PolygonRef, collection::CollectionRef,
    },
};

#[derive(Clone, Copy)]
pub enum GeometryRef<'a> {
    Point(PointRef<'a>),
    LineString(LineStringRef<'a>),
    MultiPoint(MultiPointRef<'a>),
    MultiLineString(MultiLineStringRef<'a>),
    Polygon(PolygonRef<'a>),
    MultiPolygon(MultiPolygonRef<'a>),
    Collection(CollectionRef<'a>),
}

impl<'a> FromBytes<'a, &'a [u8]> for GeometryRef<'a> {
    fn parse(
        reader: &mut udled::Reader<'_, 'a, &'a [u8]>,
        byteorder: udled::bytes::Endian,
    ) -> udled::Result<Self> {
        let ty = reader
            .peek_ch()
            .ok_or_else(|| reader.error("Expected type"))?;

        let ty = GeoType::from_u8(ty).ok_or_else(|| reader.error("Expected type"))?;

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
