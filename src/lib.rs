use std::sync::Arc;

use anyhow::Result;

use players::{get_players, Player};
use serde::{Deserialize, Serialize};
use serenity::prelude::{Mutex, TypeMapKey};
use stats::{get_stats, Stat};

pub mod commands;
mod players;
mod stats;

/// Corresponds to the customizable config file that can be modified by the user
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    experience_earned_after_success: i32,
    experience_earned_after_failure: i32,
}

impl Config {
    pub fn from(path: &str) -> Self {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }
}

/// Holds the configuration, list of stats, and player infos at all time
#[derive(Debug)]
pub struct State {
    config: Config,       // A global config
    stats: Vec<Stat>,     // The stat tree that will be used to select a stat
    players: Vec<Player>, // The player infos
}

impl TypeMapKey for State {
    type Value = Arc<Mutex<Self>>;
}

impl State {
    pub fn from_config_files() -> Result<Self> {
        let config_folder = "./config";
        Ok(State {
            config: Config::from(&format!("{}/config.json", config_folder)),
            stats: get_stats(&format!("{}/stats.txt", config_folder))?,
            players: get_players(&format!("{}/players", config_folder))?,
        })
    }
}
