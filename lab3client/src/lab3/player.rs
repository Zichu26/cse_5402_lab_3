/// player.rs
/// Author: Zichu Pan, Edgar Palomino
/// Summary: This module implements the Player structure that represents individual actors/characters in a play, 
/// managing their dialogue lines and delivery.
use std::sync::atomic::Ordering;
use super::declarations::WHINGE_MODE;
use super::script_gen::grab_trimmed_file_lines;
use std::io::Write;

pub type PlayLines = Vec<(usize, String)>; // (line_number, line_text)

pub struct Player {
    name: String,
    lines: PlayLines,
    index: usize,
}

impl Player {
    // Create a new player
    pub fn new(name: &String) -> Player {
        Player {
            name: name.clone(),
            lines: PlayLines::new(),
            index: 0,
        }
    }

    /// Parses individual script lines:
    /// - Expects format: <line_number> <dialogue_text>
    /// - Extracts line number from first token
    /// - Stores the remaining text as dialogue
    /// - Warns about invalid line numbers in whinge mode
    /// - Ignores empty lines
    fn add_script_line(&mut self, line: &String) {
        // Ignore empty lines
        if line.is_empty() {
            return;
        }

        if let Some((first_token, rest_of_line)) = line.split_once(char::is_whitespace) {
                let trimmed_rest = rest_of_line.trim(); // remove leading space
                
                // Try to parse the first token as line number
                match first_token.parse::<usize>() {
                    Ok(line_number) => {
                        self.lines.push((line_number, trimmed_rest.to_string()));
                    }
                    Err(_error_code) => {
                        if WHINGE_MODE.load(Ordering::SeqCst) {
                            writeln!(std::io::stderr().lock(), "Warning: '{}' does not represent a valid line number", first_token)
                                .expect("Failed to write to stderr");
                        }
                    }
                }
            }
    }

    /// Loads the player's script:
    /// - Reads lines from the character's script file
    /// - Parses each line using add_script_line()
    /// - Sorts lines by line number to handle out-of-order input
    pub fn prepare(&mut self, part_filename: &String) {
        let mut part_lines: Vec<String> = Vec::new();
    
        if let Err(error_code) = grab_trimmed_file_lines(part_filename, &mut part_lines) {
            std::panic::panic_any(error_code);
        }

        // Process each line and add to player's lines
        for line in &part_lines {
            self.add_script_line(line);
        }

        // Sort lines by line number to handle out-of-order lines
        self.lines.sort();
    }


    /// Delivers the next line of dialogue:
    /// - Checks if all lines have been spoken
    /// - Prints character name if speaker changes
    /// - Prints the dialogue text
    /// - Advances the index to next line
    pub fn speak(&mut self, current_speaker: &mut String) {
        // return if all lines have already been spoken
        if self.index >= self.lines.len() {
            return;
        }

        // Check if this player is different from the current speaker
        if *current_speaker != self.name {
            // Update the current speaker to this player's name
            *current_speaker = self.name.clone();
            writeln!(std::io::stdout().lock()).expect("Failed to write to stdout");
            writeln!(std::io::stdout().lock(), "{}.", self.name).expect("Failed to write to stdout");
        }

        writeln!(std::io::stdout().lock(), "{}", self.lines[self.index].1).expect("Failed to write to stdout");
        self.index += 1;
    }

    pub fn next_line(&self) -> Option<usize> {
        if self.index < self.lines.len() {
            Some(self.lines[self.index].0)
        } else {
            None
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        match (self.lines.is_empty(), other.lines.is_empty()) {
            (true, true) => true,  // Both have no lines
            (false, false) => self.lines[0].0 == other.lines[0].0,  // Both have lines, compare first line numbers
            _ => false,  // One has lines, the other doesn't
        }
    }
}

impl Eq for Player {}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.lines.is_empty(), other.lines.is_empty()) {
            (true, true) => std::cmp::Ordering::Equal,  // Both have no lines
            (true, false) => std::cmp::Ordering::Less,  // Self has no lines, other does
            (false, true) => std::cmp::Ordering::Greater,  // Self has lines, other doesn't
            (false, false) => self.lines[0].0.cmp(&other.lines[0].0),  // Compare first line numbers
        }
    }
}

