use core::fmt::{self, Write as _};

use geo::{
    BoundingRect, Centroid, ChamberlainDuquetteArea, Contains, Distance, GeodesicArea, Haversine,
    Intersects, Within,
};
use geo_traits::to_geo::{ToGeoGeometry, ToGeoPoint};
use geob::{GeoType, Geob, types::GeometryRef};
use rusqlite::{Connection, Error, Result, functions::FunctionFlags};

use crate::template::{Lookup, replace};

const COLUMN_TRIGGER: &str = include_str!("column_trigger.sql");

pub fn register_functions(conn: &Connection) -> Result<bool> {
    conn.create_scalar_function(
        "ST_FromText",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            //

            let text: String = ctx.get(0)?;

            let output =
                Geob::from_text(&text).map_err(|err| Error::UserFunctionError(err.into()))?;

            Ok(output)
        },
    )?;

    conn.create_scalar_function("ST_ToText", 1, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
        let text: Geob = ctx.get(0)?;

        let output = text.to_string();

        Ok(output)
    })?;

    conn.create_scalar_function(
        "ST_GetSRID",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let text: Geob = ctx.get(0)?;

            let output: u32 = text.srid().into();

            Ok(output)
        },
    )?;

    conn.create_scalar_function(
        "ST_GetType",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let geo: Geob = ctx.get(0)?;
            Ok(geo.kind().to_string())
        },
    )?;

    #[cfg(feature = "proj")]
    conn.create_scalar_function(
        "ST_Transform",
        2,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let mut text: Geob = ctx.get(0)?;
            let srid: u32 = ctx.get(1)?;

            let text_srid: u32 = text.srid().into();

            if srid == text_srid {
                return Ok(text);
            }

            text.project(srid);

            Ok(text)
        },
    )?;

    conn.create_scalar_function(
        "ST_Distance",
        2,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let a: Geob = ctx.get(0)?;
            let b: Geob = ctx.get(1)?;

            if a.kind() != GeoType::Point {
            } else if b.kind() != GeoType::Point {
            }

            if a.srid() != b.srid() {}

            let a = a.geometry();
            let b = b.geometry();

            let distance = match (a, b) {
                (GeometryRef::Point(a), GeometryRef::Point(b)) => {
                    let a = a.to_point();
                    let b = b.to_point();

                    let ditance = Haversine.distance(a, b);
                    ditance
                }
                _ => {
                    todo!()
                }
            };

            Ok(distance)
        },
    )?;

    conn.create_scalar_function("ST_Within", 2, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
        let a: Geob = ctx.get(0)?;
        let b: Geob = ctx.get(1)?;

        if a.srid() != b.srid() {}

        let a = a.geometry().to_geometry();
        let b = b.geometry().to_geometry();

        Ok(a.is_within(&b))
    })?;

    conn.create_scalar_function(
        "ST_Contains",
        2,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let a: Geob = ctx.get(0)?;
            let b: Geob = ctx.get(1)?;

            if a.srid() != b.srid() {}

            let a = a.geometry().to_geometry();
            let b = b.geometry().to_geometry();

            Ok(a.contains(&b))
        },
    )?;

    conn.create_scalar_function(
        "ST_Intersects",
        2,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let a: Geob = ctx.get(0)?;
            let b: Geob = ctx.get(1)?;

            if a.srid() != b.srid() {
                todo!("SRID")
            }

            let a = a.geometry().to_geometry();
            let b = b.geometry().to_geometry();

            Ok(a.intersects(&b))
        },
    )?;

    conn.create_scalar_function(
        "ST_Envelope",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let a: Geob = ctx.get(0)?;

            let srid = a.srid();

            let a = a.geometry().to_geometry();

            let Some(out) = a.bounding_rect() else {
                todo!()
            };

            let geob = Geob::from_geo_type(&out.to_polygon(), srid);

            Ok(geob)
        },
    )?;

    conn.create_scalar_function("ST_AddColumn", 3, FunctionFlags::SQLITE_DIRECTONLY, |ctx| {
        let table: String = ctx.get(0)?;
        let column: String = ctx.get(1)?;
        let srid: u32 = ctx.get(2)?;

        let sql = replace(
            COLUMN_TRIGGER,
            &AddColumn {
                table: &table,
                column: &column,
                srid,
            },
        )
        .expect("Render");

        unsafe { ctx.get_connection()?.execute_batch(&sql) }?;

        Ok(true)
    })?;

    conn.create_scalar_function("ST_Area", 1, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
        let a: Geob = ctx.get(0)?;

        let a = a.geometry().to_geometry();

        let area = a.geodesic_area_unsigned();

        Ok(area)
    })?;

    conn.create_scalar_function("ST_Area", 2, FunctionFlags::SQLITE_DETERMINISTIC, |ctx| {
        let a: Geob = ctx.get(0)?;
        let accurate: bool = ctx.get(1)?;

        let a = a.geometry().to_geometry();

        let area = if accurate {
            a.geodesic_area_signed()
        } else {
            a.chamberlain_duquette_signed_area()
        };

        Ok(area)
    })?;

    conn.create_scalar_function(
        "ST_Perimeter",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let a: Geob = ctx.get(0)?;

            let a = a.geometry().to_geometry();

            let area = a.geodesic_perimeter();

            Ok(area)
        },
    )?;

    conn.create_scalar_function(
        "ST_Centroid",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let geo: Geob = ctx.get(0)?;

            let a = geo.geometry().to_geometry();

            let area = a.centroid();

            Ok(area.map(|point| Geob::new_point(geo.srid(), point.x(), point.y()).unwrap()))
        },
    )?;

    Ok(true)
}

struct AddColumn<'a> {
    table: &'a str,
    column: &'a str,
    srid: u32,
}

impl<'a> Lookup for AddColumn<'a> {
    fn replace(&self, name: &str, output: &mut String) -> core::fmt::Result {
        match name {
            "name" => {
                write!(output, "{}_{}_geob_trigger", self.table, self.column)?;
            }
            "table" => {
                output.push_str(self.table);
            }
            "column" => {
                output.push_str(self.column);
            }
            "srid" => {
                write!(output, "{}", self.srid)?;
            }
            _ => return Err(fmt::Error),
        }

        Ok(())
    }
}
