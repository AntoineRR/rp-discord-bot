use std::collections::HashMap;

use config::players::get_players;
use config::stat::Stat;
use config::Config;
use config::{affinity::Affinity, parser::TreeStructure};
use tracing::info;

use crate::config::parser::get_tree;
use crate::config::players::Player;

pub mod commands;
mod config;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, State, Error>;

/// Holds the configuration, list of stats, and player infos at all time
#[derive(Debug)]
pub struct State {
    pub config: Config,               // A global config
    stats: Vec<Stat>,                 // The stat tree that will be used to select a stat
    affinities: Vec<Affinity>,        // The available affinities groups
    players: HashMap<String, String>, // The mapping of a discord name with a player file name
}

impl State {
    pub fn from_config_files() -> Result<Self, Error> {
        let config_folder = "./config";
        info!("Loading config from {config_folder}");
        let config = Config::from(&format!("{config_folder}/config.json"))?;
        let stats = get_tree(&format!("{config_folder}/stats.txt"))?;
        let affinities = get_tree(&format!("{config_folder}/affinities.txt"))?;
        let players = get_players(&format!("{config_folder}/players"))?;

        check_validity(&stats, &affinities, &players)?;
        info!("Config files are correct");

        Ok(State {
            config,
            stats,
            affinities,
            players,
        })
    }
}

fn check_validity(
    stats: &[Stat],
    affinities: &[Affinity],
    players: &HashMap<String, String>,
) -> Result<(), Error> {
    info!("Checking config files coherence...");
    // Create a flat vec of stats
    let flat_stats: Vec<Stat> = stats.iter().flat_map(|s| s.flatten()).collect();

    // Check validity of affinities
    let flat_affinities: Vec<Affinity> = affinities.iter().flat_map(|a| a.flatten()).collect();
    for affinity in flat_affinities {
        if !flat_stats.iter().any(|s| s.id == affinity.id) {
            return Err(format!(
                "Affinity stat {:?} is not in stat file",
                affinity.display_name,
            )
            .into());
        }
    }

    // Check validity of each player
    for file_path in players.values() {
        let player = Player::from(file_path)?;
        for stat in player.stats.keys() {
            if !flat_stats.iter().any(|s| &s.display_name == stat) {
                return Err(format!(
                    "Stat {:?} from file {} is not in stat file",
                    stat, file_path
                )
                .into());
            }
        }
        for stat in &flat_stats {
            if !player.stats.iter().any(|(s, _)| s == &stat.display_name) {
                return Err(
                    format!("Stat {:?} is not in file {}", stat.display_name, file_path).into(),
                );
            }
        }
        for major_affinity in player.affinities.major {
            if !affinities.iter().any(|a| a.display_name == major_affinity) {
                return Err(format!(
                    "Major affinity {:?} from file {} is not in stat file",
                    major_affinity, file_path
                )
                .into());
            }
        }
        for minor_affinity in player.affinities.minor {
            if !affinities.iter().any(|a| a.display_name == minor_affinity) {
                return Err(format!(
                    "Minor affinity {:?} from file {} is not in stat file",
                    minor_affinity, file_path
                )
                .into());
            }
        }
        for talent in player.talents {
            if !flat_stats.iter().any(|s| s.display_name == talent) {
                return Err(format!(
                    "Talent {:?} from file {} is not in stat file",
                    talent, file_path
                )
                .into());
            }
        }
    }

    Ok(())
}
