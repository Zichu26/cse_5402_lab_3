/// return_wrapper.rs
/// Author: Zichu Pan, Edgar Palomino
use std::process::{ExitCode, Termination};
use std::io::Write;

pub const SUCCESS: u8 = 0;

pub struct ReturnWrapper {
    code: u8,
}

impl ReturnWrapper {
    pub fn new(code: u8) -> ReturnWrapper {
        ReturnWrapper { code }
    }
}

impl Termination for ReturnWrapper {
    fn report(self) -> ExitCode {
        if self.code != SUCCESS {
            writeln!(std::io::stderr().lock(), "Error: {}", self.code).expect("Failed to write to stderr");
        }
        ExitCode::from(self.code)
    }
}