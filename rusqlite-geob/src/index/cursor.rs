use std::{marker::PhantomData, mem::transmute};

use geob::Geob;
use rusqlite::{ffi, vtab::VTabCursor};

use crate::index::{tree::Query, types::QueryPlanFlags};

use super::SpartialIndex;

#[repr(C)]
pub struct SpartialIndexCursor<'vtab> {
    base: ffi::sqlite3_vtab_cursor,
    phantom: PhantomData<&'vtab SpartialIndex>,
    iter: Option<Box<dyn Iterator<Item = (u64, Geob)> + 'vtab>>,
    next: Option<(u64, Geob)>,
}

impl<'vtab> SpartialIndexCursor<'vtab> {
    pub fn new() -> SpartialIndexCursor<'vtab> {
        SpartialIndexCursor {
            base: ffi::sqlite3_vtab_cursor::default(),
            phantom: PhantomData,
            iter: None,
            next: None,
        }
    }

    /// Accessor to the associated virtual table.
    fn vtab(&self) -> &SpartialIndex {
        unsafe { &*(self.base.pVtab as *const SpartialIndex) }
    }
}

unsafe impl<'vtab> VTabCursor for SpartialIndexCursor<'vtab> {
    fn filter(
        &mut self,
        idx_num: std::ffi::c_int,
        idx_str: Option<&str>,
        args: &rusqlite::vtab::Filters<'_>,
    ) -> rusqlite::Result<()> {
        let idx_num = QueryPlanFlags::from_bits_truncate(idx_num);
        let mut i = 0;

        let mut query = Query::default();

        if idx_num.contains(QueryPlanFlags::DISTANCE_EQ) {
            query.distance_eq = Some(args.get(i)?);
            i += 1;
        }

        if idx_num.contains(QueryPlanFlags::DISTANCE_LT) {
            query.distance_lt = Some(args.get(i)?);
            i += 1;
        }

        if idx_num.contains(QueryPlanFlags::GEOMETRY_EQ) {
            query.geometry_eq = Some(args.get(i)?);
            i += 1;
        }

        if idx_num.contains(QueryPlanFlags::GEMETRY_IN) {
            query.geometry_match = Some(args.get(i)?);
            i += 1;
        }

        if idx_num.contains(QueryPlanFlags::ID_EQ) {
            query.id_eq = Some(args.get(i)?);
            i += 1;
        }

        self.iter = Some(unsafe { transmute(self.vtab().tree.select(self.vtab().srid, query)?) });
        self.next = self.iter.as_mut().unwrap().next();
        Ok(())
    }

    fn next(&mut self) -> rusqlite::Result<()> {
        self.next = self.iter.as_mut().and_then(|m| m.next());
        Ok(())
    }

    fn eof(&self) -> bool {
        self.next.is_none()
    }

    fn column(
        &self,
        ctx: &mut rusqlite::vtab::Context,
        i: std::ffi::c_int,
    ) -> rusqlite::Result<()> {
        let Some(next) = &self.next else {
            return Ok(());
        };

        match i {
            0 => {
                ctx.set_result(&next.0)?;
            }

            1 => {
                ctx.set_result(&next.1)?;
            }
            2 => {}
            _ => {
                panic!("Inalid column");
            }
        }
        Ok(())
    }

    fn rowid(&self) -> rusqlite::Result<i64> {
        Ok(self.next.as_ref().map(|m| m.0).unwrap_or_default() as _)
    }
}
