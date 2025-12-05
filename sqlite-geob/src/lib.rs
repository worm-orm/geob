use std::os::raw::{c_char, c_int};

use rusqlite::ffi;
use rusqlite::{Connection, Result};

/// Entry point for SQLite to load the extension.
/// See <https://sqlite.org/c3ref/load_extension.html> on this function's name and usage.
/// # Safety
/// This function is called by SQLite and must be safe to call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3_extension_init(
    db: *mut ffi::sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    unsafe { Connection::extension_init2(db, pz_err_msg, p_api, extension_init) }
}

fn extension_init(db: Connection) -> Result<bool> {
    rusqlite_geob::register(&db)
}
