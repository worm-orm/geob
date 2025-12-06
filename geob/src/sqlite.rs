use rusqlite::{
    Result, ToSql,
    types::{FromSql, FromSqlError, Value, ValueRef},
};

use crate::{Geob, types::GeobRef};

impl ToSql for Geob {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(ValueRef::Blob(
            self.as_ref(),
        )))
    }
}

impl FromSql for Geob {
    fn column_result(value: ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            ValueRef::Text(text) => {
                let text =
                    core::str::from_utf8(text).map_err(|err| FromSqlError::Other(err.into()))?;
                Geob::from_text(text).map_err(|err| FromSqlError::Other(err.into()))
            }
            ValueRef::Blob(blob) => {
                Geob::from_bytes(blob).map_err(|err| FromSqlError::Other(err.into()))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl From<Geob> for Value {
    fn from(value: Geob) -> Self {
        Value::Blob(value.as_ref().to_vec())
    }
}

impl<'a> From<&'a Geob> for ValueRef<'a> {
    fn from(value: &'a Geob) -> Self {
        ValueRef::Blob(value.as_ref())
    }
}

impl<'a> From<GeobRef<'a>> for ValueRef<'a> {
    fn from(value: GeobRef<'a>) -> Self {
        ValueRef::Blob(value.bytes)
    }
}
