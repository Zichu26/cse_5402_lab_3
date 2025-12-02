/// main.rs
/// Lab 3 - CSE 5402 Fall 2025
/// Author: Zichu Pan, Edgar Palomino
/// Summary: Simple test client for validating the lab3server

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::ExitCode;
use std::time::Duration;

const EXPECTED_ARGS: usize = 3;  // program_name, address, token
const PROGRAM_NAME_INDEX: usize = 0;
const ADDRESS_INDEX: usize = 1;
const TOKEN_INDEX: usize = 2;
const SUCCESS: u8 = 0;
const BAD_COMMAND_LINE: u8 = 1;
const FAILED_CONNECTION: u8 = 2;
const FAILED_SENT: u8 = 3;

fn usage(program_name: &str) {
    println!("usage: {} <network_address> <token>", program_name);
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != EXPECTED_ARGS {
        usage(&args[PROGRAM_NAME_INDEX]);
        return ExitCode::from(BAD_COMMAND_LINE);
    }
    let address = &args[ADDRESS_INDEX];
    let token = &args[TOKEN_INDEX];

    // Connect to the server
    let mut stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to connect to '{}': {}", address, e);
            return ExitCode::from(FAILED_CONNECTION);
        }
    };

    // Send the token to the server 
    if let Err(e) = writeln!(stream, "{}", token) {
        eprintln!("Error: Failed to send token to server: {}", e);
        return ExitCode::from(FAILED_SENT);
    }
    if let Err(e) = stream.flush() {
        eprintln!("Error: Failed to flush stream: {}", e);
        return ExitCode::from(FAILED_SENT);
    }

    if token == "quit" {
        let duration = Duration::from_secs(1);
        std::thread::sleep(duration);
        // Connect again to wake up the server from the accept call
        let _ = TcpStream::connect(address);
        return ExitCode::from(SUCCESS);
    }

    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        match line {
            Ok(text) => {
                println!("{}", text);
            }
            Err(_) => {
                break;
            }
        }
    }

    ExitCode::from(SUCCESS)
}