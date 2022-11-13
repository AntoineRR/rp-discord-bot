use anyhow::Result;

use super::parser::{clean_input, TreeStructure};

/// Represent a stat tree node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
    pub id: String,
    pub display_name: String,
    pub sub_stats: Vec<Stat>,
}

impl TreeStructure for Stat {
    fn get_children(&self) -> &[Self] {
        &self.sub_stats
    }

    /// Create a Stat from a raw line of the file
    /// The raw input will be cleaned to be used as an id for the stat
    fn from_line(raw_line: &str, sub_stats: &[Stat]) -> Result<Self> {
        if sub_stats.len() > 20 {
            return Err(
                "There shouldn't be more than 20 stats in one category, check your stats.txt file.",
            )
            .map_err(anyhow::Error::msg);
        }

        Ok(Stat {
            id: raw_line.trim().chars().map(clean_input).collect(),
            display_name: raw_line.trim().to_string(),
            sub_stats: sub_stats.to_vec(),
        })
    }
}
