/// script_gen.rs
/// Author: Zichu Pan, Edgar Palomino
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::net::TcpStream;

use super::declarations::{FAILED_TO_OPEN_FILE, FAILED_TO_READ_LINE_FROM_FILE};

pub fn get_buffered_reader(source: &String) -> Result<Box<dyn BufRead>, u8> {
    // Check if it's a network path: "net:IP:PORT:filename"
    if source.starts_with("net:") {
        // net:127.0.0.1:7777:filename.txt
        let after_net = &source[4..];
        // "127.0.0.1:7777:filename.txt"
        // Split into IP, PORT, and filename
        let parts: Vec<&str> = after_net.splitn(3, ':').collect();
        
        if parts.len() < 3 {
            let _ = writeln!(std::io::stderr().lock(), 
                "Error: Invalid network path format: '{}' (expected net:IP:PORT:filename)", source);
            return Err(FAILED_TO_OPEN_FILE);
        }
        
        let ip = parts[0];
        let port = parts[1];
        let filename = parts[2];
        let address = format!("{}:{}", ip, port);
        
        // Connect to the server
        let mut stream = match TcpStream::connect(&address) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(std::io::stderr().lock(), 
                    "Error: Failed to connect to '{}': {}", address, e);
                return Err(FAILED_TO_OPEN_FILE);
            }
        };
        
        // Send the filename to the server
        if let Err(e) = writeln!(stream, "{}", filename) {
            let _ = writeln!(std::io::stderr().lock(), 
                "Error: Failed to send filename '{}' to server: {}", filename, e);
            return Err(FAILED_TO_OPEN_FILE);
        }
        if let Err(e) = stream.flush() {
            let _ = writeln!(std::io::stderr().lock(), 
                "Error: Failed to flush stream: {}", e);
            return Err(FAILED_TO_OPEN_FILE);
        }
        
        Ok(Box::new(BufReader::new(stream)))
    } else {
        let file = match File::open(source) {
            Ok(f) => f,
            Err(e) => {
                let _ = writeln!(std::io::stderr().lock(), 
                    "Error: Failed to open file '{}': {}", source, e);
                return Err(FAILED_TO_OPEN_FILE);
            }
        };
        
        Ok(Box::new(BufReader::new(file)))
    }
}

pub fn grab_trimmed_file_lines(filename: &String, lines: &mut Vec<String>) -> Result<(), u8> {
    // The core function used for extracting data from files
    // Used for both reading the config file line by line and reading the parts file line by line
    let mut reader = match get_buffered_reader(filename) {
        Ok(r) => r,
        Err(error_code) => {
            return Err(error_code);
        }
    };
    let mut line = String::new();
    
    loop {
        line.clear();
        
        let bytes_read = match reader.read_line(&mut line) {
            Ok(bytes) => bytes,
            Err(error_code) => {
                writeln!(std::io::stderr().lock(), "Error: Failed to read line from file '{}': {}", filename, error_code)
                    .expect("Failed to write to stderr");
                return Err(FAILED_TO_READ_LINE_FROM_FILE);
            }
        };
        
        if bytes_read == 0 {
            return Ok(());
        }

        lines.push(line.trim().to_string());
    }
}


    