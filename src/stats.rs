use std::fs;

use anyhow::Result;

fn clean_input(c: char) -> char {
    let c = c.to_lowercase().next().unwrap();
    match c {
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'à' | 'â' => 'a',
        'ï' => 'i',
        'ô' => 'o',
        'œ' => 'e',
        ' ' | '-' | '/' => '_',
        c => c,
    }
}

/// Represent a stat tree node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
    pub id: String,
    pub display_name: String,
    pub sub_stats: Vec<Stat>,
}

impl Stat {
    /// Create a Stat from a raw line of the file
    /// The raw input will be cleaned to be used as an id for the stat
    pub fn from(raw_line: &str, sub_stats: &[Stat]) -> Result<Self> {
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

struct ParsedLine {
    value: String,
    indent_level: usize,
}

impl ParsedLine {
    fn root() -> Self {
        ParsedLine {
            value: "root".to_string(),
            indent_level: 0,
        }
    }

    fn from(line: &str) -> Self {
        ParsedLine {
            value: line.trim().to_string(),
            indent_level: get_indent_level(line) + 4, // +4 because 0 indent level is the root node
        }
    }
}

// Get how indented a line is, in number of spaces
fn get_indent_level(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| match c {
            '\t' => 4,
            ' ' => 1,
            _ => 1,
        })
        .sum()
}

// Build a stat tree based on the stats as lines, by parsing the line's indentation
fn build_stat_tree(lines: &[ParsedLine], index: usize) -> Result<Stat> {
    if index + 1 >= lines.len() {
        return Stat::from(&lines[index].value, &[]);
    }
    let mut children: Vec<Stat> = vec![];
    let current_indent_level = lines[index].indent_level;
    let children_indent_level = current_indent_level + 4;
    for (idx, line) in lines[index + 1..].iter().enumerate() {
        match line.indent_level {
            i if i == children_indent_level => {
                children.push(build_stat_tree(lines, index + 1 + idx)?)
            }
            i if i < children_indent_level => return Stat::from(&lines[index].value, &children),
            _ => (),
        }
    }
    Stat::from(&lines[index].value, &children)
}

/// Get the stat tree from the stats.txt file
pub fn get_stats(path: &str) -> Result<Vec<Stat>> {
    let file_content = fs::read_to_string(path).expect("Could not read stats file");
    // A root node is needed to build the Stat tree
    let mut parsed_lines = vec![ParsedLine::root()];
    parsed_lines.append(
        &mut file_content
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(ParsedLine::from)
            .collect(),
    );
    // Drop the root node by returning only its children
    Ok(build_stat_tree(&parsed_lines, 0)?.sub_stats)
}

#[cfg(test)]
mod tests {
    use crate::stats::Stat;

    use super::{build_stat_tree, ParsedLine};

    fn assert_vec_eq<T: PartialEq>(vec1: Vec<T>, vec2: Vec<T>) {
        assert!(vec1.iter().zip(vec2).all(|(v1, v2)| *v1 == v2));
    }

    fn get_parsed_lines(lines: &[&str]) -> Vec<ParsedLine> {
        let mut parsed_lines = vec![ParsedLine::root()];
        parsed_lines.extend(
            lines
                .iter()
                .map(|line| ParsedLine::from(line))
                .collect::<Vec<ParsedLine>>(),
        );
        parsed_lines
    }

    #[test]
    fn parse_stats_no_indent() {
        let lines = ["Stat1", "Stat2", "Stat3"];
        let result = build_stat_tree(&get_parsed_lines(&lines), 0)
            .unwrap()
            .sub_stats;
        let expected = vec![
            Stat::from("Stat1", &[]).unwrap(),
            Stat::from("Stat2", &[]).unwrap(),
            Stat::from("Stat3", &[]).unwrap(),
        ];
        assert_vec_eq(result, expected);
    }

    #[test]
    fn parse_stats_one_level_indent() {
        let lines = ["Stat1", "    Stat2", "    Stat3"];
        let result = build_stat_tree(&get_parsed_lines(&lines), 0)
            .unwrap()
            .sub_stats;
        let expected = vec![Stat::from(
            "Stat1",
            &[
                Stat::from("Stat2", &[]).unwrap(),
                Stat::from("Stat3", &[]).unwrap(),
            ],
        )
        .unwrap()];
        assert_vec_eq(result, expected);
    }

    #[test]
    fn parse_stats_multiple_one_level_indent() {
        let lines = [
            "Stat1",
            "    Stat2",
            "    Stat3",
            "Stat4",
            "    Stat5",
            "    Stat6",
        ];
        let result = build_stat_tree(&get_parsed_lines(&lines), 0)
            .unwrap()
            .sub_stats;
        let expected = vec![
            Stat::from(
                "Stat1",
                &[
                    Stat::from("Stat2", &[]).unwrap(),
                    Stat::from("Stat3", &[]).unwrap(),
                ],
            )
            .unwrap(),
            Stat::from(
                "Stat4",
                &[
                    Stat::from("Stat5", &[]).unwrap(),
                    Stat::from("Stat6", &[]).unwrap(),
                ],
            )
            .unwrap(),
        ];
        assert_vec_eq(result, expected);
    }

    #[test]
    fn parse_stats_complex_indent() {
        let lines = [
            "Stat1",
            "    Stat2",
            "        Stat3",
            "Stat4",
            "    Stat5",
            "        Stat6",
            "        Stat7",
            "    Stat8",
            "    Stat9",
            "        Stat10",
        ];
        let result = build_stat_tree(&get_parsed_lines(&lines), 0)
            .unwrap()
            .sub_stats;
        let expected = vec![
            Stat::from(
                "Stat1",
                &[Stat::from("Stat2", &[Stat::from("Stat3", &[]).unwrap()]).unwrap()],
            )
            .unwrap(),
            Stat::from(
                "Stat4",
                &[
                    Stat::from(
                        "Stat5",
                        &[
                            Stat::from("Stat6", &[]).unwrap(),
                            Stat::from("Stat7", &[]).unwrap(),
                        ],
                    )
                    .unwrap(),
                    Stat::from("Stat8", &[]).unwrap(),
                    Stat::from("Stat9", &[Stat::from("Stat10", &[]).unwrap()]).unwrap(),
                ],
            )
            .unwrap(),
        ];
        assert_vec_eq(result, expected);
    }
}
