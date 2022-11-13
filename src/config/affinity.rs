use anyhow::Result;

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
