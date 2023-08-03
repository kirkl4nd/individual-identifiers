use rand::Rng;
use reqwest::Error;
use serde_json::Value;
use uuid::Uuid;

pub enum Identifier {
    Default {
        uuid: Uuid,
    },
    Success {
        uuid: Uuid,
        name: String,
    },
    Failure {
        uuid: Uuid,
        error: String,
    },
}

impl Identifier {
    pub fn new() -> Self {
        Self::Default { uuid: Uuid::new_v4() }
    }
}