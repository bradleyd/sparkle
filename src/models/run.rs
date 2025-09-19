use chrono::Utc;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub id: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub dry_run: Option<i64>,
    pub error_msg: Option<String>,
}

impl Run {
    pub fn create(conn: &Connection, dry_run: i64, error_msg: Option<&str>) -> Result<Run> {
        conn.execute(
            "INSERT INTO runs(dry_run,error_msg) VALUES (?1, ?2)",
            params![dry_run, error_msg.unwrap_or_default()],
        )?;

        let run_id = conn.last_insert_rowid();
        Run::find_by_id(conn, run_id)
    }

    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Run> {
        let mut stmt = conn.prepare(
            "SELECT id, started_at, finished_at, dry_run, error_msg FROM runs WHERE id = ?1",
        )?;

        let action = stmt.query_row([id], |row| {
            Ok(Run {
                id: Some(row.get(0)?),
                started_at: row.get(1)?,
                finished_at: row.get(2)?,
                dry_run: row.get(3)?,
                error_msg: row.get(4)?,
            })
        })?;

        Ok(action)
    }

    pub fn update(conn: &Connection, id: i64) -> Result<usize> {
        let rows = conn.execute(
            "UPDATE runs set finished_at=?1 where id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;

        Ok(rows)
    }
}
