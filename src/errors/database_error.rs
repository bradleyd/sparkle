use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error), // ← The #[from] attribute is crucial!

    #[error("Not found: {entity} with {field} = {value}")]
    NotFound {
        entity: String,
        field: String,
        value: String,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Connection error: {0}")]
    Connection(String),
}
