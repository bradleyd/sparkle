use crate::DatabaseError;
use crate::Result;
use rusqlite::{Connection, params};

#[derive(Debug)]
struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "add_actions",
        sql: r#"
          CREATE TABLE IF NOT EXISTS actions (
           id           INTEGER PRIMARY KEY,
           run_id       INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
           filter_id    INTEGER NOT NULL REFERENCES filters(id),
           op           TEXT NOT NULL,             -- move | rename | copy | delete_to_trash
           src_path     TEXT,                      -- nullable for delete_to_trash restore cases
           dst_path     TEXT,                      -- nullable when skipped/failed
           created_at   TEXT NOT NULL DEFAULT current_timestamp,
           error_msg    TEXT                       -- set on failed
           );
        "#,
    },
    Migration {
        version: 2,
        name: "add_runs",
        sql: r#"
            CREATE TABLE IF NOT EXISTS runs (
              id           INTEGER PRIMARY KEY,
              started_at   TEXT NOT NULL DEFAULT current_timestamp,             -- RFC3339
              finished_at  TEXT,                      -- set at end
              error_msg    TEXT,
              dry_run      INTEGER NOT NULL DEFAULT 0 -- 0/1
            );
        "#,
    },
    Migration {
        version: 4,
        name: "add_filter",
        sql: r#"
        CREATE TABLE IF NOT EXISTS filters (
          id           INTEGER PRIMARY KEY,
          kind         TEXT NOT NULL,             -- e.g., "rename" | "move" | "delete_to_trash"
          config_json  TEXT NOT NULL,             -- frozen template/config used
          created_at   TEXT NOT NULL DEFAULT current_timestamp
        );
        "#,
    },
    Migration {
        version: 5,
        name: "add_rollbacks",
        sql: r#"
            CREATE TABLE IF NOT EXISTS rollbacks (
               id           INTEGER PRIMARY KEY,
               rb_op        TEXT NOT NULL,             -- move_back | remove_created | restore_from_trash
               rb_from      TEXT,                      -- e.g., current dst
               rb_to        TEXT,                      -- e.g., original src
               rb_details   TEXT                       -- optional JSON
             );
        "#,
    },
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_schema_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            println!(
                "Running migration {}: {}",
                migration.version, migration.name
            );

            let tx = conn.unchecked_transaction().map_err(DatabaseError::from)?;

            tx.execute_batch(migration.sql).map_err(|e| {
                DatabaseError::Migration(format!(
                    "Failed to execute migration {}: {}",
                    migration.version, e
                ))
            })?;

            set_schema_version(&tx, migration.version)?;

            tx.commit().map_err(DatabaseError::from)?;

            println!("Migration {} completed successfully", migration.version);
        }
    }

    Ok(())
}

fn get_schema_version(conn: &Connection) -> Result<i32> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER)",
        [],
    )
    .map_err(DatabaseError::from)?;

    let version: i32 = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    Ok(version)
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute("DELETE FROM schema_version", [])
        .map_err(DatabaseError::from)?;
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        params![version],
    )
    .map_err(DatabaseError::from)?;
    Ok(())
}
