use crate::FileContext;
use std::path::PathBuf;

#[derive(Debug)]
pub struct RuleError {
    pub message: String,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuleError {}

pub trait OrganizationRule: Send + Sync {
    fn name(&self) -> &str;
    fn applies_to(&self, context: &FileContext) -> RuleMatch;
    fn destination(&self, context: &FileContext) -> Result<PathBuf, RuleError>;
    fn priority(&self) -> u32;
    fn description(&self) -> &str;
}
pub enum RuleMatch {
    No,
    Yes,
    Conditional(String), // e.g., "if directory exists"
}

pub struct ContentTypeRule {
    mime_patterns: Vec<String>,
    destination_fn: Box<dyn Fn(&FileContext) -> PathBuf + Send + Sync>,
    priority: u32,
}
