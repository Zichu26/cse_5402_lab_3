/// main.rs
/// Lab 3 - CSE 5402 Fall 2025
/// Author: Zichu Pan, Edgar Palomino
/// Summary: Entry point for the multi-threaded file server.

use std::env;
use std::io::Write;

pub mod lab3;
use lab3::return_wrapper::ReturnWrapper;

use lab3::server::Server;

const EXPECTED_ARGS: usize = 2;  // program_name, address
const PROGRAM_NAME_INDEX: usize = 0;
const ADDRESS_INDEX: usize = 1;
const BAD_COMMAND_LINE_ERROR: u8 = 1;
const SUCCESS: u8 = 0;

fn usage(program_name: &str) {
    let _ = writeln!(std::io::stdout().lock(), "usage: {} <network_address>", program_name);
}

fn main() -> ReturnWrapper {
    let args: Vec<String> = env::args().collect();

    if args.len() != EXPECTED_ARGS {
        usage(&args[PROGRAM_NAME_INDEX]);
        return ReturnWrapper::new(BAD_COMMAND_LINE_ERROR);
    }

    // Run server
    let mut server = Server::new();
    if let Err(error_code) = server.open(&args[ADDRESS_INDEX]) {
        return ReturnWrapper::new(error_code);
    }
    server.run();

    ReturnWrapper::new(SUCCESS)
}