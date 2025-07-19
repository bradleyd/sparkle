use std::path::PathBuf;

pub struct Payload {
    status: bool,
    source: PathBuf,
    destination: PathBuf,
}

impl Payload {
    pub fn new() -> Self {
        Payload {
            status: false,
            source: PathBuf::new(),
            destination: PathBuf::new(),
        }
    }
}
