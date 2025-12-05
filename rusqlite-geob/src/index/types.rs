use std::ffi::c_int;

use geob::{GeoKind, GeoType};

pub const ID_IDX: c_int = 0;
pub const GEO_IDX: c_int = 1;
pub const DISTANCE_IDX: c_int = 2;

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct QueryPlanFlags: c_int {
        // start = $value  -- constraint exists
        const DISTANCE_EQ = 1 << 0;
        // stop = $value   -- constraint exists

        const DISTANCE_LT  = 1 << 1;
        const DISTANCE_LTE  = 1 << 2;
        // step = $value   -- constraint exists

        const DISTANCE_GT = 1 << 3;
        const DISTANCE_GTE = 1<< 4;

        const GEOMETRY_EQ = 1 << 5;
        const GEMETRY_IN = 1 << 6;

        const ID_EQ = 1 << 7;
        // output in descending order
        // const DESC  = 8;
        // // output in ascending order
        // const ASC  = 16;
        // Both start and stop
        // const BOTH  = QueryPlanFlags::START.bits() | QueryPlanFlags::STOP.bits();

        const ALL_DISTANCE = Self::DISTANCE_EQ.bits() | Self::DISTANCE_GT.bits() | Self::DISTANCE_GTE.bits() | Self::DISTANCE_LT.bits() | Self::DISTANCE_LTE.bits();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeometryType {
    Point = 1,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    Collection,
    Any,
}

impl From<GeoType> for GeometryType {
    fn from(value: GeoType) -> Self {
        match value {
            GeoType::Point => GeometryType::Point,
            GeoType::LineString => GeometryType::LineString,
            GeoType::Polygon => GeometryType::Polygon,
            GeoType::MultiPoint => GeometryType::MultiPoint,
            GeoType::MultiLineString => GeometryType::MultiLineString,
            GeoType::MultiPolygon => GeometryType::MultiPolygon,
            GeoType::Collection => GeometryType::Collection,
        }
    }
}

impl GeometryType {
    pub fn schema(&self) -> &str {
        match self {
            GeometryType::Point => "CREATE TABLE x(id INTEGER, geometry HIDDEN, distance HIDDEN)",
            _ => "CREATE TABLE x(id INTEGER, geometry HIDDEN)",
        }
    }

    pub fn is_valid(&self, other: GeometryType) -> bool {
        match (self, other) {
            // anything matches "Any"
            (_, GeometryType::Any) | (GeometryType::Any, _) => true,
            // Collection can contain any concrete geometry
            (GeometryType::Collection, _) => true,
            // allow a Multi* to accept its single counterpart
            (GeometryType::MultiPoint, GeometryType::Point)
            | (GeometryType::MultiLineString, GeometryType::LineString)
            | (GeometryType::MultiPolygon, GeometryType::Polygon) => true,
            // exact match
            (a, b) => a == &b,
        }
    }
}
