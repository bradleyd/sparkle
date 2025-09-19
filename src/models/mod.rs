pub mod action;
pub mod filter;
pub mod rollback;
pub mod run;

pub use action::ActionRepo;
pub use filter::FilterRepo;
pub use run::Run;

// Common traits for all models
pub trait Model {
    type Id;

    fn id(&self) -> Option<Self::Id>;
    fn set_id(&mut self, id: Self::Id);
}

pub trait Timestamps {
    fn created_at(&self) -> Option<&chrono::DateTime<chrono::Utc>>;
    fn updated_at(&self) -> Option<&chrono::DateTime<chrono::Utc>>;
}
