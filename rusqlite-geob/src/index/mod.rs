use std::num::ParseIntError;

use core::fmt::Write as _;
use geob::{Geob, SRID};
use rusqlite::{
    Connection, Error, Result,
    ffi::{self, sqlite3_vtab},
    vtab::{self, CreateVTab, IndexConstraintOp, UpdateVTab, VTab, VTabKind},
};

mod cursor;
mod tree;
mod types;

use crate::{
    index::types::{DISTANCE_IDX, GEO_IDX, ID_IDX, QueryPlanFlags},
    template::{Lookup, replace},
};

use self::{cursor::SpartialIndexCursor, tree::RStarTree};

use self::types::GeometryType;

const COLUMN_TRIGGER: &str = include_str!("column_index.sql");

pub fn register_module(conn: &Connection) -> Result<()> {
    conn.create_module(
        "SpartialIndex",
        vtab::update_module::<SpartialIndex>(),
        None,
    )
}

#[derive(Default)]
struct Options<'a> {
    table: Option<&'a str>,
    column: Option<&'a str>,
    srid: Option<u32>,
    ty: Option<GeometryType>,
    index: bool,
}

#[repr(C)]
pub struct SpartialIndex {
    base: ffi::sqlite3_vtab,
    tree: tree::RStarTree,
    name: String,
    srid: SRID,
    table: String,
    column: String,
    ty: GeometryType,
}

unsafe impl<'vtab> VTab<'vtab> for SpartialIndex {
    type Aux = ();

    type Cursor = cursor::SpartialIndexCursor<'vtab>;

    fn connect(
        db: &mut rusqlite::vtab::VTabConnection,
        _aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> rusqlite::Result<(String, Self)> {
        if args.len() < 4 {
            return Err(Error::ModuleError("no CSV file specified".to_owned()));
        }

        let mut opts = Options::default();
        opts.index = true;

        let name = str::from_utf8(args[2])
            .map_err(|err| Error::ModuleError(err.to_string()))?
            .to_string();

        let args = &args[3..];
        for c_slice in args {
            let (param, value) = rusqlite::vtab::parameter(c_slice)?;
            match param {
                "table" => {
                    opts.table = Some(value);
                }
                "column" => {
                    opts.column = Some(value);
                }
                "type" => {
                    opts.ty = Some(match value {
                        "geometry" => GeometryType::Any,
                        "point" => GeometryType::Point,
                        "linestring" => GeometryType::LineString,
                        "multilinestring" => GeometryType::MultiLineString,
                        "multipoint" => GeometryType::MultiPoint,
                        "polygon" => GeometryType::Polygon,
                        "multipolygon" => GeometryType::MultiPolygon,
                        _ => {
                            return Err(Error::ModuleError(format!(
                                "unrecognized geometry type '{param}'"
                            )));
                        }
                    });
                }
                "srid" => {
                    let srid: u32 = value
                        .parse()
                        .map_err(|err: ParseIntError| Error::ModuleError(err.to_string()))?;

                    opts.srid = Some(srid);
                }
                "index" => {
                    //
                    match value {
                        "true" => {
                            opts.index = true;
                        }
                        "false" => {
                            opts.index = false;
                        }
                        _ => {
                            return Err(Error::ModuleError(format!(
                                "unrecognized geometry type '{param}'"
                            )));
                        }
                    }
                }
                _ => {
                    return Err(Error::ModuleError(format!(
                        "unrecognized parameter '{param}'"
                    )));
                }
            }
        }

        let srid = opts
            .srid
            .ok_or_else(|| Error::ModuleError("Srid not set".to_string()))?;

        let ty = opts
            .ty
            .ok_or_else(|| Error::ModuleError("Type not set".to_string()))?;

        let table = opts
            .table
            .ok_or_else(|| Error::ModuleError("Table not set".to_string()))?
            .to_string();

        let column = opts
            .column
            .ok_or_else(|| Error::ModuleError("Column not set".to_string()))?
            .to_string();

        let mut tree = RStarTree::new(ty);

        let schema = ty.schema().to_string();

        let conn = unsafe { Connection::from_handle(db.handle())? };

        let sql = replace(
            COLUMN_TRIGGER,
            &CreateIndex {
                table: &table,
                column: &column,
                index_name: &name,
            },
        )
        .unwrap();

        conn.execute_batch(&sql)?;

        let mut stmt = conn.prepare(&format!("SELECT rowid, {column} FROM {table}"))?;
        let mut rows = stmt.query([])?;

        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            let id: u64 = row.get(0)?;
            let geo: Geob = row.get(1)?;
            items.push((id, geo));
        }

        tree.reload_batch(items)?;

        Ok((
            schema,
            SpartialIndex {
                base: sqlite3_vtab::default(),
                tree,
                name,
                srid: srid.into(),
                table,
                column,
                ty,
            },
        ))
    }

    fn best_index(&self, info: &mut rusqlite::vtab::IndexInfo) -> rusqlite::Result<()> {
        let mut idx_num = QueryPlanFlags::empty();
        let mut unusable_mask: QueryPlanFlags = QueryPlanFlags::empty();

        let mut a_idx: [Option<usize>; 5] = [None, None, None, None, None];

        let num_rows = self.tree.len();
        let mut est_cost = 0.;

        for (i, c) in info.constraints().enumerate() {
            let (i_col, i_mask) = if c.column() == DISTANCE_IDX {
                let i_mask = match c.operator() {
                    IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ => {
                        (0, QueryPlanFlags::DISTANCE_EQ)
                    }
                    IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_LT => {
                        (1, QueryPlanFlags::DISTANCE_LT)
                    }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_GT => {
                    //     (1, QueryPlanFlags::DISTANCE_GT)
                    // }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_LE => {
                    //     (2, QueryPlanFlags::DISTANCE_LTE)
                    // }

                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_GE => {
                    //     (4, QueryPlanFlags::DISTANCE_GTE)
                    // }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_NE => todo!(),
                    _ => {
                        panic!("Unsupported constraint")
                    }
                };

                est_cost += 1000.;

                i_mask
            } else if c.column() == GEO_IDX {
                let i_mast = match c.operator() {
                    IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ => {
                        (2, QueryPlanFlags::GEOMETRY_EQ)
                    }

                    IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_MATCH => {
                        (3, QueryPlanFlags::GEMETRY_IN)
                    }

                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_GT => {
                    //     (1, QueryPlanFlags::DISTANCE_GT)
                    // }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_LE => {
                    //     (2, QueryPlanFlags::DISTANCE_LTE)
                    // }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_LT => {
                    //     (3, QueryPlanFlags::DISTANCE_LT)
                    // }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_GE => {
                    //     (4, QueryPlanFlags::DISTANCE_GTE)
                    // }
                    // IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_NE => todo!(),
                    _ => {
                        panic!("Unsupported constraint")
                    }
                };

                est_cost += 600.;

                i_mast
            } else if c.column() == ID_IDX {
                let i_mast = match c.operator() {
                    IndexConstraintOp::SQLITE_INDEX_CONSTRAINT_EQ => (4, QueryPlanFlags::ID_EQ),

                    _ => {
                        panic!("Unsupported constraint")
                    }
                };

                est_cost += 600.;

                i_mast
            } else {
                todo!()
            };

            if !c.is_usable() {
                unusable_mask |= i_mask;
            } else {
                idx_num |= i_mask;
                a_idx[i_col] = Some(i);
            }
        }

        if idx_num.intersects(QueryPlanFlags::ALL_DISTANCE)
            && !idx_num.contains(QueryPlanFlags::GEOMETRY_EQ)
        {
            return Err(Error::ModuleError(
                "Needs a geometry for this query".to_string(),
            ));
        }

        let mut n_arg = 0;
        for j in a_idx.iter().flatten() {
            n_arg += 1;
            let mut constraint_usage = info.constraint_usage(*j);
            constraint_usage.set_argv_index(n_arg);
            constraint_usage.set_omit(true);
        }
        if !(unusable_mask & !idx_num).is_empty() {
            return Err(Error::ModuleError("Constraint error".to_string()));
        }

        info.set_idx_num(idx_num.bits());

        info.set_estimated_cost(est_cost);
        info.set_estimated_rows(num_rows as _);

        Ok(())
    }

    fn open(&'vtab mut self) -> rusqlite::Result<Self::Cursor> {
        Ok(SpartialIndexCursor::new())
    }
}

impl<'vtab> CreateVTab<'vtab> for SpartialIndex {
    const KIND: rusqlite::vtab::VTabKind = VTabKind::Default;
}

impl<'vtab> UpdateVTab<'vtab> for SpartialIndex {
    fn delete(&mut self, arg: rusqlite::types::ValueRef<'_>) -> Result<()> {
        let id = arg.as_i64()?;

        let _ = self.tree.remove(id as _);

        Ok(())
    }

    fn insert(&mut self, args: &vtab::Inserts<'_>) -> Result<i64> {
        let id: u64 = args.get(2)?;
        let geo: Option<Geob> = args.get(3)?;

        let Some(geo) = geo else { return Ok(0) };

        self.tree.insert(id, geo)?;

        Ok(0)
    }

    fn update(&mut self, args: &vtab::Updates<'_>) -> Result<()> {
        let rowid: u64 = args.get(0)?;
        let geob: Geob = args.get(3)?;

        let _ = self.tree.remove(rowid as _);
        self.tree.insert(rowid, geob)?;

        Ok(())
    }
}

struct CreateIndex<'a> {
    table: &'a str,
    column: &'a str,
    index_name: &'a str,
}

impl<'a> Lookup for CreateIndex<'a> {
    fn replace(&self, name: &str, output: &mut String) -> core::fmt::Result {
        match name {
            "name" => {
                write!(output, "{}_{}_index_geob_trigger", self.table, self.column)?;
            }
            "index" => {
                write!(output, "{}", self.index_name)?;
            }
            "table" => {
                output.push_str(self.table);
            }
            "column" => {
                output.push_str(self.column);
            }
            _ => return Err(core::fmt::Error),
        }

        Ok(())
    }
}
