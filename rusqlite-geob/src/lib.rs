use rusqlite::{Connection, Result};

mod functions;
#[cfg(feature = "index")]
mod index;
mod template;

pub fn register(conn: &Connection) -> Result<bool> {
    functions::register_functions(conn)?;
    #[cfg(feature = "index")]
    index::register_module(conn)?;

    Ok(true)
}
