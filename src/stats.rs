use std::{
    collections::HashMap,
    fs::{self, read_dir},
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
    pub id: String,
    pub display_name: String,
    pub sub_stats: Vec<Stat>,
}

impl Stat {
    /// Create a Stat from a raw line of the file
    /// The raw input will be cleaned to be used as an id for the stat
    pub fn from(raw_line: &str, sub_stats: &[Stat]) -> Self {
        if sub_stats.len() > 25 {
            panic!(
                "There shouldn't be more than 25 stats in one category, check your stats.txt file."
            )
        }

        Stat {
            id: raw_line.trim().chars().map(clean_input).collect(),
            display_name: raw_line.trim().to_string(),
            sub_stats: sub_stats.to_vec(),
        }
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
fn build_stat_tree(lines: &[ParsedLine], index: usize) -> Stat {
    if index + 1 >= lines.len() {
        return Stat::from(&lines[index].value, &[]);
    }
    let mut children: Vec<Stat> = vec![];
    let current_indent_level = lines[index].indent_level;
    let children_indent_level = current_indent_level + 4;
    for (idx, line) in lines[index + 1..].iter().enumerate() {
        match line.indent_level {
            i if i == children_indent_level => {
                children.push(build_stat_tree(lines, index + 1 + idx))
            }
            i if i < children_indent_level => return Stat::from(&lines[index].value, &children),
            _ => (),
        }
    }
    Stat::from(&lines[index].value, &children)
}

/// Get the stat tree from the stats.txt file
pub fn get_stats() -> Vec<Stat> {
    let file_content = fs::read_to_string("./stats.txt").expect("Could not read stats file");
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
    build_stat_tree(&parsed_lines, 0).sub_stats
}

#[derive(Debug, PartialEq, Eq)]
pub struct Player {
    pub name: String,
    pub discord_name: String,
    pub stats: HashMap<String, i32>,
}

impl Player {
    pub fn from(name: &str, discord_name: &str, stats: HashMap<String, i32>) -> Self {
        Player {
            name: name.to_string(),
            discord_name: discord_name.to_string(),
            stats,
        }
    }
}

fn parse_player(name: &str, lines: &[&str]) -> Player {
    // TODO: check if all stats of the player are in the stat tree
    let parse_line = |line: &str| {
        let splitted: Vec<&str> = line.split(':').collect();
        if splitted.len() != 2 {
            panic!("Syntax error at line {line}");
        }
        let name = splitted[0].trim().chars().map(clean_input).collect();
        let value: i32 = splitted[1]
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("Syntax error at line {line}"));
        (name, value)
    };

    if lines.len() < 2 {
        panic!("Please provide at least a discord name and one stat in your {name}.txt file");
    }
    let discord_name = lines[0].trim();
    let parsed_stats = lines[1..].iter().map(|line| parse_line(*line)).collect();
    Player::from(name, discord_name, parsed_stats)
}

pub fn get_players() -> Vec<Player> {
    let player_paths = read_dir("./players").expect("You should have a 'players' directory");
    player_paths
        .map(|p| {
            let path = p.as_ref().unwrap().path();
            let file_name = path.file_stem().unwrap();
            let raw = fs::read_to_string(&path).unwrap();
            let lines: Vec<&str> = raw.split('\n').filter(|line| !line.is_empty()).collect();
            parse_player(file_name.to_str().unwrap(), &lines)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::stats::Stat;

    use super::{build_stat_tree, parse_player, ParsedLine, Player};

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
        let result = build_stat_tree(&get_parsed_lines(&lines), 0).sub_stats;
        let expected = vec![
            Stat::from("Stat1", &[]),
            Stat::from("Stat2", &[]),
            Stat::from("Stat3", &[]),
        ];
        assert_vec_eq(result, expected);
    }

    #[test]
    fn parse_stats_one_level_indent() {
        let lines = ["Stat1", "    Stat2", "    Stat3"];
        let result = build_stat_tree(&get_parsed_lines(&lines), 0).sub_stats;
        let expected = vec![Stat::from(
            "Stat1",
            &[Stat::from("Stat2", &[]), Stat::from("Stat3", &[])],
        )];
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
        let result = build_stat_tree(&get_parsed_lines(&lines), 0).sub_stats;
        let expected = vec![
            Stat::from(
                "Stat1",
                &[Stat::from("Stat2", &[]), Stat::from("Stat3", &[])],
            ),
            Stat::from(
                "Stat4",
                &[Stat::from("Stat5", &[]), Stat::from("Stat6", &[])],
            ),
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
        let result = build_stat_tree(&get_parsed_lines(&lines), 0).sub_stats;
        let expected = vec![
            Stat::from("Stat1", &[Stat::from("Stat2", &[Stat::from("Stat3", &[])])]),
            Stat::from(
                "Stat4",
                &[
                    Stat::from(
                        "Stat5",
                        &[Stat::from("Stat6", &[]), Stat::from("Stat7", &[])],
                    ),
                    Stat::from("Stat8", &[]),
                    Stat::from("Stat9", &[Stat::from("Stat10", &[])]),
                ],
            ),
        ];
        println!("{:?}", result);
        assert_vec_eq(result, expected);
    }

    #[test]
    fn parse_player_stats() {
        let name = "Player1";
        let lines = ["DiscordName1", "Stat1: 12", "Stat2: 5", "Stat3: 128"];
        let mut parsed_stats = HashMap::new();
        parsed_stats.insert("stat1".to_string(), 12);
        parsed_stats.insert("stat2".to_string(), 5);
        parsed_stats.insert("stat3".to_string(), 128);
        let result = parse_player(&name, &lines);
        let expected = Player::from(name, "DiscordName1", parsed_stats);
        assert_eq!(result, expected);
    }
}
