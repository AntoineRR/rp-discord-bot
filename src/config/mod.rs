use serde::{Deserialize, Serialize};

pub mod affinity;
pub mod parser;
pub mod players;
pub mod stat;

/// Corresponds to the customizable config file that can be modified by the user
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub experience_earned_after_success: i32,
    pub experience_earned_after_failure: i32,
}

impl Config {
    pub fn from(path: &str) -> Self {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }
}
