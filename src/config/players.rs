use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_dir};

use crate::Error;

use super::affinity::{Affinities, Affinity};

// https://stackoverflow.com/questions/67789198/how-can-i-sort-fields-in-alphabetic-order-when-serializing-with-serde
// values get sorted because serde_json uses a BTreeMap internally
fn sort_alphabetically<T: Serialize, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    value.serialize(serializer)
}

// A wrapper struct to sort fields alphabetically when serializing
#[derive(Serialize)]
struct SortAlphabetically<T: Serialize>(#[serde(serialize_with = "sort_alphabetically")] T);

/// Describe a player
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    #[serde(skip)]
    path: String, // The path to the file representing this player
    pub name: String,                    // The name of the player in the game
    pub discord_name: String,            // The discord pseudo of the player
    pub stats: HashMap<String, i32>, // The stats of the player along with his experience for each of them
    pub affinities: Affinities,      // The affinities of the player
    pub talents: Vec<String>,        // The talents of the player (+20% on exp)
    pub modifiers: HashMap<String, i32>, // The permanent bonus and malus of the player (due to armors for example)
}

impl Player {
    /// Create a Player from its representation file
    pub fn from(path: &str) -> Result<Self, Error> {
        let mut value: Player = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        value.path = path.to_string();
        Ok(value)
    }

    /// Increase the experience of the player in the given stat by the given amount
    pub fn increase_experience(&mut self, exp_to_add: i32, stat_name: &str) -> Result<(), Error> {
        self.stats
            .entry(stat_name.to_string())
            .and_modify(|value| *value += exp_to_add);
        let to_save = serde_json::to_string_pretty(&SortAlphabetically(&self))?;
        std::fs::write(&self.path, to_save)?;
        Ok(())
    }

    /// Is the provided stat a talent of this player?
    pub fn is_talent(&self, stat: &str) -> bool {
        self.talents.iter().any(|t| t == stat)
    }

    /// Is the provided affinity a major affinity?
    pub fn is_major_affinity(&self, stat: &str, affinity_list: &[Affinity]) -> Result<bool, Error> {
        self.affinities.is_major(stat, affinity_list)
    }

    /// Is the provided affinity a minor affinity?
    pub fn is_minor_affinity(&self, stat: &str, affinity_list: &[Affinity]) -> Result<bool, Error> {
        self.affinities.is_minor(stat, affinity_list)
    }

    /// Get the bonus / malus for this stat
    pub fn get_modifier(&self, stat: &str) -> i32 {
        match self.modifiers.get(stat) {
            Some(m) => *m,
            None => 0,
        }
    }
}

/// Parse and get the players from the "players" folder
pub fn get_players(path: &str) -> Result<HashMap<String, String>, Error> {
    let player_paths = read_dir(path)?;

    Ok(player_paths
        .map(|p| {
            let path = p.as_ref().unwrap().path();
            let path_str = path.as_os_str().to_str().unwrap();
            Player::from(path_str)
        })
        .collect::<Result<Vec<Player>, Error>>()?
        .iter()
        .map(|p| (p.discord_name.to_string(), p.path.to_string()))
        .collect())
}
