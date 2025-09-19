use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRepo {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub filter_id: Option<i64>,
    pub op: String,
    pub src_path: String,
    pub dst_path: String,
    pub created_at: Option<String>,
    pub error_msg: Option<String>,
}

impl ActionRepo {
    pub fn new(action: ActionRepo) -> Self {
        Self {
            id: None,
            run_id: action.run_id,
            filter_id: action.filter_id,
            op: action.op,
            src_path: action.src_path,
            dst_path: action.dst_path,
            created_at: None,
            error_msg: None,
        }
    }

    pub fn create(conn: &Connection, action: ActionRepo) -> Result<ActionRepo> {
        conn.execute(
            "INSERT INTO actions(run_id, filter_id, op, src_path, dst_path) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![action.run_id, action.filter_id, action.op, action.src_path, action.dst_path ],
        )?;

        let action_id = conn.last_insert_rowid();
        ActionRepo::find_by_id(conn, action_id)
    }

    pub fn find_by_id(conn: &Connection, id: i64) -> Result<ActionRepo> {
        let mut stmt = conn.prepare(
            "SELECT id, run_id, filter_id, created_at, op, src_path, dst_path, error_msg FROM actions WHERE id = ?1",
        )?;

        let action = stmt.query_row([id], |row| {
            Ok(ActionRepo {
                id: Some(row.get(0)?),
                run_id: row.get(1)?,
                filter_id: row.get(2)?,
                created_at: row.get(3)?,
                op: row.get(4)?,
                src_path: row.get(5)?,
                dst_path: row.get(6)?,
                error_msg: row.get(7)?,
            })
        })?;

        Ok(action)
    }
}
