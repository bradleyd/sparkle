use crate::Result;
use crate::errors::DatabaseError;
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;

pub struct DatabaseConnection {
    conn: Connection,
}

impl AsRef<Connection> for DatabaseConnection {
    fn as_ref(&self) -> &Connection {
        &self.conn
    }
}

impl DatabaseConnection {
    pub fn new_in_memory() -> crate::Result<Self> {
        let conn = Connection::open_in_memory().map_err(DatabaseError::from)?;
        Ok(Self { conn })
    }

    pub fn new_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path).map_err(DatabaseError::from)?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn close(self) -> SqliteResult<()> {
        self.conn.close().map_err(|(_, err)| err)
    }

    pub fn initialize(&self) -> Result<()> {
        super::migrations::run_migrations(&self.conn)?;
        Ok(())
    }
}

impl std::ops::Deref for DatabaseConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
