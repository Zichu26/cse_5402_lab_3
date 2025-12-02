/// declarations.rs
/// Author: Zichu Pan, Edgar Palomino
/// Summary: Defines constants, exit codes and global state 
use std::sync::atomic::AtomicBool;

pub const MIN_ARGS: usize = 2;  // program_name script
pub const MAX_ARGS: usize = 3;  // program_name script WHINGE_MODE
pub const PROGRAM_NAME_INDEX: usize = 0;
pub const CONFIG_FILE_INDEX: usize = 1;
pub const VERBOSE_FLAG_INDEX: usize = 2;

// exit codes
pub const BAD_COMMAND_LINE_ERROR: u8 = 1;  
pub const FAILED_TO_OPEN_FILE: u8 = 2;
pub const SCRIPT_PARSING_ERROR: u8 = 3;
pub const CONFIG_PARSING_ERROR: u8 = 4;
pub const FAILED_TO_READ_LINE_FROM_FILE: u8 = 5;
pub const SUCCESS: u8 = 0;  

pub static WHINGE_MODE: AtomicBool = AtomicBool::new(false);


