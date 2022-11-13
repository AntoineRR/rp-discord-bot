use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::parser::{clean_input, TreeStructure};

/// Represent an affinity with its name and stats included in this affinity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affinity {
    pub id: String,
    pub display_name: String,
    pub stats: Vec<Affinity>,
}

impl TreeStructure for Affinity {
    fn get_children(&self) -> &[Self] {
        &self.stats
    }

    /// Create an Affinity from a raw line of the file
    /// The raw input will be cleaned to be used as an id for the affinity
    fn from_line(raw_line: &str, stats: &[Affinity]) -> Result<Self> {
        Ok(Affinity {
            id: raw_line.trim().chars().map(clean_input).collect(),
            display_name: raw_line.trim().to_string(),
            stats: stats.to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affinities {
    pub major: Vec<String>, // The major affinities (+10% on exp)
    pub minor: Vec<String>, // The minor affinities (+5% on exp)
}

impl Affinities {
    pub fn is_major(&self, stat: &str, affinity_list: &[Affinity]) -> Result<bool> {
        for name in &self.major {
            if is_stat_in_affinity(stat, name, affinity_list)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_minor(&self, stat: &str, affinity_list: &[Affinity]) -> Result<bool> {
        for name in &self.minor {
            if is_stat_in_affinity(stat, name, affinity_list)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn is_stat_in_affinity(
    stat: &str,
    affinity_name: &str,
    affinity_list: &[Affinity],
) -> Result<bool> {
    let affinity = affinity_list
        .iter()
        .find(|&a| a.display_name == affinity_name)
        .context("Affinity not found in the list of affinities")?;
    if affinity.stats.is_empty() {
        Ok(affinity.display_name == stat)
    } else {
        Ok(affinity.stats.iter().any(|a| a.display_name == stat))
    }
}
