/// play.rs
/// Author: Zichu Pan, Edgar Palomino
/// Summary: This module implements the core Play structure that orchestrates a performance by managing scene fragments.
use std::sync::atomic::Ordering;
use std::io::Write;
use std::sync::{Arc, Mutex};
use super::scene_fragment::SceneFragment;
use super::declarations::{WHINGE_MODE, SCRIPT_PARSING_ERROR};
use super::script_gen::grab_trimmed_file_lines;

pub type ScriptConfig = Vec<(bool, String)>;
pub type Fragments = Vec<Arc<Mutex<SceneFragment>>>;

const CONFIG_FILENAME_INDEX: usize = 0;
const CONFIG_SCRIPT_LENGTH: usize = 1;
const SCENE_SCRIPT_INDEX: usize = 0;
const SCENE_SCRIPT_LENGTH: usize = 2;

pub struct Play {
    fragments: Fragments,
}

impl Play {
    pub fn new() -> Play {
        Play {
            fragments: Vec::new(),
        }
    }

    /// Converts the ScriptConfig into SceneFragment objects:
    /// - Scene titles are stored temporarily
    /// - Config filenames trigger creation of new fragments with the current title
    pub fn process_config(&mut self, config: &ScriptConfig) -> Result<(), u8> {
        let mut title = String::new();
        let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
        
        for config_entry in config {
            match config_entry {
                (is_scene_title, text) => {
                    if *is_scene_title {
                        // Update the title string
                        title = text.clone();
                    } else {
                        // Create the fragment and wrap in Arc<Mutex<>>
                        let fragment = SceneFragment::new(&title);
                        title = String::new();
                        let fragment_arc = Arc::new(Mutex::new(fragment));
                        
                        // Add the fragment to the play
                        self.fragments.push(Arc::clone(&fragment_arc));
                        
                        // Clone the config filename for the thread
                        let config_filename = text.clone();
                        
                        // Spawn a thread to call prepare
                        let handle = std::thread::spawn(move || {
                            match fragment_arc.lock() {
                                Ok(ref mut frag) => {
                                    frag.prepare(&config_filename);
                                }
                                Err(_) => {
                                    std::panic::panic_any(SCRIPT_PARSING_ERROR);
                                }
                            }
                        });
                        
                        handles.push(handle);
                    }
                }
            }
        }

        for handle in handles {
            match handle.join() {
                Ok(_) => {}
                Err(_) => {
                    return Err(SCRIPT_PARSING_ERROR);
                }
            }
        }

        Ok(())
    }

    /// Processes individual lines:
    /// - Lines starting with [scene] are treated as scene titles
    /// - Other non-blank lines are treated as configuration filenames
    /// - Warns about missing scene titles or extra tokens (in whinge mode)
    fn add_config(&mut self, line: &String, config: &mut ScriptConfig) {
        // Ignore blank lines
        if line.trim().is_empty() {
            return;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        
        if tokens[SCENE_SCRIPT_INDEX] == "[scene]" {
            // Case 1: [scene] title
            if tokens.len() == SCENE_SCRIPT_LENGTH - 1 {
                // No scene title provided
                if WHINGE_MODE.load(Ordering::SeqCst) {
                    writeln!(std::io::stderr().lock(), "Warning: [scene] without a scene title")
                        .expect("Failed to write to stderr"); 
                }
                return;
            } else {
                // Concatenate remaining tokens as scene title
                let scene_title = tokens[1..].join(" ");
                config.push((true, scene_title));
            }
        } else {
            // Case 2: config filename
            let config_filename = tokens[CONFIG_FILENAME_INDEX].to_string();
            config.push((false, config_filename));
            
            if tokens.len() > CONFIG_SCRIPT_LENGTH && WHINGE_MODE.load(Ordering::SeqCst) {
                writeln!(std::io::stderr().lock(), "Warning: Extra tokens after configuration file name: '{}'", 
                         tokens[1..].join(" ")).expect("Failed to write to stderr"); 
            }
        }
    }

    /// Parses the script file line-by-line into a ScriptConfig
    pub fn read_config(&mut self, script_filename: &String, config: &mut ScriptConfig) -> Result<(), u8> {
        let mut script_lines: Vec<String> = Vec::new();
        
        if let Err(error_code) = grab_trimmed_file_lines(script_filename, &mut script_lines) {
            return Err(error_code);
        }

        if script_lines.is_empty() {
            writeln!(std::io::stderr().lock(), "Error: Script file '{}' contains no lines", script_filename)
                .expect("Failed to write to stderr");
            return Err(SCRIPT_PARSING_ERROR);
        }
        
        for line in &script_lines {
            self.add_config(line, config);
        }
        
        Ok(())
    }

    /// Main entry point that:
    /// - Reads the script configuration file
    /// - Parses it into scene fragments
    /// - Validates that at least one fragment exists and the first has a title
    pub fn prepare(&mut self, script_filename: &String) -> Result<(), u8> {
        let mut config: ScriptConfig = Vec::new();
        
        if let Err(error_code) = self.read_config(script_filename, &mut config) {
            return Err(error_code);
        }
        
        if let Err(error_code) = self.process_config(&config) {
            return Err(error_code);
        }

        if self.fragments.is_empty() {
            writeln!(std::io::stderr().lock(), "Error: No scene fragments were created").expect("Failed to write to stderr");
            return Err(SCRIPT_PARSING_ERROR);
        }
        
        // Lock the first fragment to check if it has a title
        match self.fragments[0].lock() {
            Ok(ref fragment) => {
                if !fragment.has_title() {
                    writeln!(std::io::stderr().lock(), "Error: First fragment must have a title")
                        .expect("Failed to write to stderr");
                    return Err(SCRIPT_PARSING_ERROR);
                }
            }
            Err(_) => {
                writeln!(std::io::stderr().lock(), "Error: Failed to lock first fragment")
                    .expect("Failed to write to stderr");
                return Err(SCRIPT_PARSING_ERROR);
            }
        }
        
        Ok(())
    }

    ///  Executes the play:
    /// - Handles player entrances 
    /// - Each fragment recites its lines
    /// - Handles player exits 
    pub fn recite(&mut self) {
        let num_fragments = self.fragments.len();
        
        for i in 0..num_fragments {
            if i == 0 {
                // First fragment
                match self.fragments[i].lock() {
                    Ok(ref mut fragment) => {
                        fragment.enter_all();
                    }
                    Err(_) => {
                        writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {}", i)
                            .expect("Failed to write to stderr");
                        continue;
                    }
                }
            } else {
                // Lock both current and previous fragments for enter()
                match self.fragments[i - 1].lock() {
                    Ok(ref prev_fragment) => {
                        match self.fragments[i].lock() {
                            Ok(ref mut curr_fragment) => {
                                curr_fragment.enter(&prev_fragment);
                            }
                            Err(_) => {
                                writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {}", i)
                                    .expect("Failed to write to stderr");
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {}", i - 1)
                            .expect("Failed to write to stderr");
                        continue;
                    }
                }
            }
            
            match self.fragments[i].lock() {
                Ok(ref mut fragment) => {
                    fragment.recite();
                }
                Err(_) => {
                    writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {} for recite", i)
                        .expect("Failed to write to stderr");
                }
            }
            
            writeln!(std::io::stdout().lock()).expect("Failed to write to stdout");

            if i == num_fragments - 1 {
                // Final fragment
                match self.fragments[i].lock() {
                    Ok(ref fragment) => {
                        fragment.exit_all();
                    }
                    Err(_) => {
                        writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {}", i)
                            .expect("Failed to write to stderr");
                    }
                }
            } else {
                // Lock both current and next fragments for exit()
                match self.fragments[i + 1].lock() {
                    Ok(ref next_fragment) => {
                        match self.fragments[i].lock() {
                            Ok(ref curr_fragment) => {
                                curr_fragment.exit(&next_fragment);
                            }
                            Err(_) => {
                                let _ = writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {}", i)
                                    .expect("Failed to write to stderr");
                            }
                        }
                    }
                    Err(_) => {
                        writeln!(std::io::stderr().lock(), "Error: Failed to lock fragment {}", i + 1)
                            .expect("Failed to write to stderr");
                    }
                }
            }
        }
    }
}