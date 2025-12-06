#![no_std]

extern crate alloc;

// mod binary;
mod geob;
#[cfg(feature = "sqlite")]
mod sqlite;
pub mod srid;
pub mod types;
mod util;
pub mod wkt;
mod writer;

#[cfg(feature = "rstar")]
pub mod rstar;

#[cfg(feature = "geo-traits")]
mod geotypes;
#[cfg(feature = "proj")]
mod projection;

pub use self::{
    // binary::GeoType,
    geob::Geob,
    srid::{EPSG, SRID},
    types::{GeoType, GeobRef},
};
