use serde::{Deserialize, Serialize};

use crate::Error;

pub mod affinity;
pub mod parser;
pub mod players;
pub mod stat;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "law", content = "parameters")]
pub enum StatisticLaw {
    Uniform,
    Normal(f64, f64),
}

/// Corresponds to the customizable config file that can be modified by the user
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub game_master_discord_name: String,
    pub experience_earned_after_success: i32,
    pub experience_earned_after_failure: i32,
    pub learning_constant: f64,
    pub talent_increase_percentage: f64,
    pub major_affinity_increase_percentage: f64,
    pub minor_affinity_increase_percentage: f64,
    pub roll_command_statistic_law: StatisticLaw,
}

impl Config {
    pub fn from(path: &str) -> Result<Self, Error> {
        let result = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        Ok(result)
    }
}
