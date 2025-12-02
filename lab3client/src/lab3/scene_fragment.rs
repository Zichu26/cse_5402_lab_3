/// scene_fragment.rs
/// Author: Zichu Pan, Edgar Palomino
/// Summary: This module  implements the SceneFragment structure that represents individual scenes within a play, 
/// managing players (actors) and their dialogue.
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::io::Write;
use super::player::Player;
use super::declarations::{WHINGE_MODE, CONFIG_PARSING_ERROR};
use super::script_gen::grab_trimmed_file_lines;

pub type PlayConfig = Vec<(String, String)>; // (part_name, part_filename)
      
pub const PART_NAME_INDEX: usize = 0;
pub const PART_FILENAME_INDEX: usize = 1;
pub const CONFIG_LINE_TOKEN_COUNT: usize = 2;

pub struct SceneFragment {
    title: String,
    players: Vec<Arc<Mutex<Player>>>,
}

impl SceneFragment {
    pub fn new(title: &String) -> SceneFragment {
        SceneFragment {
            title: title.clone(),
            players: Vec::new(),
        }
    }

    /// Instantiates Player objects:
    /// - Creates a Player for each character
    /// - Calls prepare() on each player with their script file
    pub fn process_config(&mut self, config: &PlayConfig) {
        let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
    
        for config_entry in config {
            match config_entry {
                (part_name, part_filename) => {
                    // Create a new Player instance using the part name
                    let player = Player::new(part_name);
                    
                    // Wrap in Arc<Mutex<>> and add to players vector
                    let player_arc = Arc::new(Mutex::new(player));
                    self.players.push(Arc::clone(&player_arc));
                    
                    // Clone the part filename for the thread
                    let filename = part_filename.clone();
                    
                    // Spawn a thread to call prepare
                    let handle = std::thread::spawn(move || {
                        match player_arc.lock() {
                            Ok(ref mut p) => {
                                p.prepare(&filename);
                            }
                            Err(_) => {
                                std::panic::panic_any(super::declarations::FAILED_TO_OPEN_FILE);
                            }
                        }
                    });
                    
                    handles.push(handle);
                }
            }
        }
        
        // Join all threads - unwrap will propagate any panics
        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn add_config(&mut self, line: &String, config: &mut PlayConfig) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        if tokens.len() != CONFIG_LINE_TOKEN_COUNT {
            if WHINGE_MODE.load(Ordering::SeqCst) {
                if tokens.len() < CONFIG_LINE_TOKEN_COUNT {
                    writeln!(std::io::stderr().lock(), "Warning: Configuration line has too few tokens (expected {}, got {}): '{}'",
                             CONFIG_LINE_TOKEN_COUNT, tokens.len(), line).expect("Failed to write to stderr");
                } else {
                    writeln!(std::io::stderr().lock(), "Warning: Configuration line has too many tokens (expected {}, got {}): '{}'", 
                             CONFIG_LINE_TOKEN_COUNT, tokens.len(), line).expect("Failed to write to stderr");
                }
            }
        }
        
        if tokens.len() >= CONFIG_LINE_TOKEN_COUNT {
            config.push((
                tokens[PART_NAME_INDEX].to_string(),
                tokens[PART_FILENAME_INDEX].to_string()
            ));
        }
    }

    /// Parse configuration files:
    /// - Each line should have exactly 2 tokens: character name and their script file
    /// - Warns about malformed lines (too few/many tokens) in whinge mode
    /// - Builds a PlayConfig with character-to-script mappings
    pub fn read_config(&mut self, config_filename: &String, config: &mut PlayConfig) -> Result<(), u8> {
        let mut config_lines: Vec<String> = Vec::new();
        
        if let Err(error_code) = grab_trimmed_file_lines(config_filename, &mut config_lines) {
            return Err(error_code);
        }

        if config_lines.is_empty() {
            writeln!(std::io::stderr().lock(), "Error: Config file '{}' contains no lines", config_filename)
                .expect("Failed to write to stderr");
            return Err(CONFIG_PARSING_ERROR);
        }
        
        for line in &config_lines {
            self.add_config(line, config);
        }
        
        Ok(())
    }

    /// Comparison function for Arc<Mutex<Player>> references.
    /// Returns Equal if either lock fails, otherwise compares the underlying Players.
    fn compare_players(a: &Arc<Mutex<Player>>, b: &Arc<Mutex<Player>>) -> std::cmp::Ordering {
        match a.lock() {
            Ok(ref player_a) => {
                match b.lock() {
                    Ok(ref player_b) => {
                        match Player::partial_cmp(player_a, player_b) {
                            Some(ordering) => ordering,
                            None => std::cmp::Ordering::Equal,
                        }
                    }
                    Err(_) => std::cmp::Ordering::Equal,
                }
            }
            Err(_) => std::cmp::Ordering::Equal,
        }
    }

    /// Main setup method that:
    /// - Reads the configuration file for this scene
    /// - Creates and prepares Player objects for each character
    /// - Sorts players by line number
    pub fn prepare(&mut self, config_filename: &String) {
        let mut config: PlayConfig = Vec::new();
        
        if let Err(error_code) = self.read_config(config_filename, &mut config) {
            std::panic::panic_any(error_code);
        }
        
        self.process_config(&config);

        self.players.sort_by(Self::compare_players);
    }

    pub fn has_title(&self) -> bool {
        !self.title.trim().is_empty()
    }

    fn print_title(&self, is_first: bool) {
        if !self.title.trim().is_empty() {
            if !is_first {
                // Blank line before scene title (except first)
                writeln!(std::io::stdout().lock()).expect("Failed to write to stdout"); 
            }
        writeln!(std::io::stdout().lock(), "{}", self.title).expect("Failed to write to stdout"); 
        writeln!(std::io::stdout().lock()).expect("Failed to write to stdout"); 
        }
    }

    pub fn enter(&self, previous: &SceneFragment) {
        self.print_title(false);
        for player_arc in &self.players {
            match player_arc.lock() {
                Ok(ref player) => {
                    // Check if player was in previous scene
                    let in_previous = previous.players.iter().any(|p| {
                        match p.lock() {
                            Ok(ref prev_player) => prev_player.name() == player.name(),
                            Err(_) => false,
                        }
                    });
                    if !in_previous {
                        writeln!(std::io::stdout().lock(), "[Enter {}.]", player.name())
                            .expect("Failed to write to stdout"); 
                    }
                }
                Err(_) => {
                    writeln!(std::io::stderr().lock(), "Error: Failed to lock player in enter()")
                        .expect("Failed to write to stderr"); 
                }
            }
        }
    }

    pub fn enter_all(&self) {    
        self.print_title(true);
        for player_arc in &self.players {
            match player_arc.lock() {
                Ok(ref player) => {
                    writeln!(std::io::stdout().lock(), "[Enter {}.]", player.name()).expect("Failed to write to stdout"); 
                }
                Err(_) => {
                    writeln!(std::io::stderr().lock(), "Error: Failed to lock player in enter_all()")
                        .expect("Failed to write to stderr"); 
                }
            }
        }
    }

    pub fn exit(&self, next: &SceneFragment) {
        for player_arc in self.players.iter().rev() {
            match player_arc.lock() {
                Ok(ref player) => {
                    // Check if this player will be in next scene
                    let in_next = next.players.iter().any(|p| {
                        match p.lock() {
                            Ok(ref next_player) => next_player.name() == player.name(),
                            Err(_) => false,
                        }
                    });
                    if !in_next {
                        writeln!(std::io::stdout().lock(), "[Exit {}.]", player.name()).expect("Failed to write to stdout"); 
                    }
                }
                Err(_) => {
                    writeln!(std::io::stderr().lock(), "Error: Failed to lock player in exit()").expect("Failed to write to stderr"); 
                }
            }
        }
    }

    pub fn exit_all(&self) {
        for player_arc in self.players.iter().rev() {
            match player_arc.lock() {
                Ok(ref player) => {
                    writeln!(std::io::stdout().lock(), "[Exit {}.]", player.name()).expect("Failed to write to stdout"); 
                }
                Err(_) => {
                    writeln!(std::io::stderr().lock(), "Error: Failed to lock player in exit_all()").expect("Failed to write to stderr"); 
                }
            }
        }
    }

    /// Orchestrates dialogue delivery:
    /// - Repeatedly finds the player with the smallest next line number
    /// - That player speaks their line
    /// - Tracks expected line numbers to detect missing/duplicate lines
    /// - Warns about line number issues in whinge mode
    /// - Continues until all players have delivered all lines
    pub fn recite(&mut self) {
        let mut current_speaker = String::new();
        let mut expected_line_number: usize = 0;
        
        loop {
            // Find the player with the smallest next line number
            let mut next_line_number: Option<usize> = None;
            let mut next_player_index: Option<usize> = None;
            for (index, player_arc) in self.players.iter().enumerate() {
                match player_arc.lock() {
                    Ok(ref player) => {
                       if let Some(line_num) = player.next_line() {
                            // next_line_number is_none() means we haven't found any player with a line yet in this iteration
                            // this is the first player we've encountered who has lines remaining, 
                            // which by default is the next line number.
                            if next_line_number.is_none() || line_num < next_line_number.unwrap() {
                                next_line_number = Some(line_num);
                                next_player_index = Some(index);
                           }
                        }
                    }
                    Err(_) => {
                        writeln!(std::io::stderr().lock(), "Error: Failed to lock player {} in recite()", index)
                            .expect("Failed to write to stderr");
                    }
                }
            }
            // If no player has lines left, we're done
            if next_player_index.is_none() {
                break;
            }
            
            // Check for missing line numbers
            let actual_line_number = next_line_number.unwrap();
            if actual_line_number > expected_line_number {
                if WHINGE_MODE.load(Ordering::SeqCst) {
                    for missing in expected_line_number..actual_line_number {
                        writeln!(std::io::stderr().lock(), "Warning: Missing line number {}", missing)
                            .expect("Failed to write to stderr"); 
                    }
                }
                expected_line_number = actual_line_number;
            }
            
            // Check for duplicate line numbers
            if actual_line_number == expected_line_number {
                // This is the expected line, advance the counter
                expected_line_number += 1;
            } else if actual_line_number < expected_line_number {
                // This is a duplicate
                if WHINGE_MODE.load(Ordering::SeqCst) {
                    let _ = writeln!(std::io::stderr().lock(), "Warning: Duplicate line number {}", actual_line_number)
                        .expect("Failed to write to stderr"); 
                }
            }
            
            // Have the selected player speak their line
            let player_index = next_player_index.unwrap();
            match self.players[player_index].lock() {
                Ok(ref mut player) => {
                    player.speak(&mut current_speaker);
                }
                Err(_) => {
                    writeln!(std::io::stderr().lock(), "Error: Failed to lock player {} for speaking", player_index)
                        .expect("Failed to write to stderr"); 
               }
            }
        }
    }
}