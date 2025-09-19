pub mod cli;
pub mod config;
pub mod content_info;
pub mod crawl;
pub mod database;
pub mod errors;
pub mod file_detector;
pub mod file_metadata;
pub mod handlers;
pub mod models;
pub mod utils;

pub use database::connection::DatabaseConnection;
pub use errors::database_error::DatabaseError;
pub use models::action::ActionRepo;
pub use models::filter::FilterRepo;

pub type Result<T> = std::result::Result<T, DatabaseError>;
