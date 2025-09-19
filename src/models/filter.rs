use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterRepo {
    pub id: Option<i64>,
    pub kind: String,
    pub config_json: String,
    pub created_at: Option<String>,
}

impl FilterRepo {
    pub fn create(conn: &Connection, kind: &str, config_json: &str) -> Result<FilterRepo> {
        let _ = conn.execute(
            "INSERT INTO filters(kind, config_json) VALUES (?1, ?2)",
            params![kind, config_json],
        )?;

        let filter_id = conn.last_insert_rowid();
        tracing::debug!("Filter id from create: {}", filter_id);
        FilterRepo::find_by_id(conn, filter_id)
    }

    pub fn find_by_id(conn: &Connection, id: i64) -> Result<FilterRepo> {
        let mut stmt =
            conn.prepare("SELECT id, kind, config_json, created_at FROM filters WHERE id = ?1")?;

        let filter = stmt.query_row([id], |row| {
            Ok(FilterRepo {
                id: row.get(0)?,
                kind: row.get(1)?,
                config_json: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        Ok(filter)
    }

    pub fn all(conn: &Connection) -> Result<Vec<FilterRepo>> {
        let mut stmt = conn.prepare("SELECT * FROM filters;")?;

        let rows = stmt.query_map([], |row| {
            Ok(FilterRepo {
                id: row.get(0)?,
                kind: row.get(1)?,
                config_json: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut filters = Vec::new();
        for row in rows {
            filters.push(row?);
        }

        Ok(filters)
    }
}
