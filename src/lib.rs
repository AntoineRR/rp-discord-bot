use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use config::affinity::Affinity;
use config::players::get_players;
use config::stat::Stat;
use config::Config;
use serenity::prelude::{Mutex, TypeMapKey};
use tracing::info;

use crate::config::parser::get_tree;

pub mod commands;
mod config;

/// Holds the configuration, list of stats, and player infos at all time
#[derive(Debug)]
pub struct State {
    config: Config,   // A global config
    stats: Vec<Stat>, // The stat tree that will be used to select a stat
    #[allow(dead_code)]
    affinities: Vec<Affinity>, // The available affinities groups
    players: HashMap<String, String>, // The mapping of a discord name with a player file name
}

impl TypeMapKey for State {
    type Value = Arc<Mutex<Self>>;
}

impl State {
    pub fn from_config_files() -> Result<Self> {
        let config_folder = "./config";
        info!("Loading config from {}", config_folder);
        Ok(State {
            config: Config::from(&format!("{}/config.json", config_folder)),
            stats: get_tree(&format!("{}/stats.txt", config_folder))?,
            affinities: get_tree(&format!("{}/affinities.txt", config_folder))?,
            players: get_players(&format!("{}/players", config_folder))?,
        })
    }
}
