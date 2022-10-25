use std::fs;

#[derive(Debug, Clone)]
pub struct Stat {
    pub id: String,
    pub display_name: String,
    pub sub_stats: Vec<Stat>,
}

impl Stat {
    /// Create a Stat from a raw line of the file
    /// The raw input will be cleaned to be used as an id for the stat
    pub fn from(raw_line: &str, sub_stats: &[Stat]) -> Self {
        let clean_fn = |c: char| {
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
        };

        Stat {
            id: raw_line.trim().chars().map(clean_fn).collect(),
            display_name: raw_line.trim().to_string(),
            sub_stats: sub_stats.to_vec(),
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
// TODO: fix the last entry of the list not being in the correct node
// TODO: limit the node at the same level to 25 (discord limit for the number of buttons on a message)
fn build_stat_tree(lines: &[&str], index: &mut usize, indent_level: usize) -> Vec<Stat> {
    let current_line = lines[*index];
    if *index < lines.len() - 1 {
        *index += 1;
        let next_line = lines[*index];
        let next_indent_level = get_indent_level(next_line);
        if next_indent_level > indent_level + 4 {
            panic!("Invalid stat file format");
        } else if next_indent_level == indent_level + 4 {
            let current_stat = Stat::from(
                current_line,
                &build_stat_tree(lines, index, next_indent_level),
            );
            return vec![current_stat]
                .into_iter()
                .chain(build_stat_tree(lines, index, indent_level))
                .collect::<Vec<Stat>>();
        } else if next_indent_level == indent_level {
            let current_stat = Stat::from(current_line, &[]);
            return vec![current_stat]
                .into_iter()
                .chain(build_stat_tree(lines, index, indent_level))
                .collect();
        }
    }
    vec![Stat::from(current_line, &[])]
}

/// Get the stat tree from the stats.txt file
pub fn get_stats() -> Vec<Stat> {
    let file_content = fs::read_to_string("./stats.txt").expect("Could not read stats file");
    let lines: Vec<&str> = file_content
        .split('\n')
        .filter(|line| !line.is_empty())
        .collect();
    build_stat_tree(&lines, &mut 0, 0)
}
