/// server.rs
/// Lab 3 - CSE 5402 Fall 2025
/// Author: Zichu Pan, Edgar Palomino
/// Summary: Multi-threaded file server implementation.
use std::io::Write;
use std::net::TcpListener;
use std::io::{BufRead, BufReader, Read};
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};

static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

pub const FAILED_TO_BIND: u8 = 2;

pub struct Server {
    listener: Option<TcpListener>,
    listening_addr: String,
}

impl Server {
    /// Creates a new Server with no listener bound
    pub fn new() -> Server {
        Server {
            listener: None,
            listening_addr: String::new(),
        }
    }

    /// Returns true if the server has an active listener, false otherwise
    pub fn is_open(&self) -> bool {
        self.listener.is_some()
    }

    /// Binds the server to the given address
    pub fn open(&mut self, addr: &str) -> Result<(), u8> {
        match TcpListener::bind(addr) {
            Ok(tcp_listener) => {
                self.listener = Some(tcp_listener);
                self.listening_addr = addr.to_string();
                Ok(())
            }
            Err(e) => {
                let _ = writeln!(std::io::stderr().lock(), "Error: Failed to bind to '{}': {}", addr, e);
                Err(FAILED_TO_BIND)
            }
        }
    }

    /// Main server loop - accepts connections and spawns handler threads
    pub fn run(&mut self) {
        while !CANCEL_FLAG.load(Ordering::SeqCst) && self.listener.is_some() {
            if let Some(ref listener) = self.listener {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        // Check CANCEL_FLAG again immediately after accept
                        if CANCEL_FLAG.load(Ordering::SeqCst) {
                            return;
                        }
                        
                        std::thread::spawn(move || {
                            Self::handle_connection(stream);
                        });
                    }
                    Err(e) => {
                        // Check CANCEL_FLAG immediately
                        if CANCEL_FLAG.load(Ordering::SeqCst) {
                            return;
                        }
                        let _ = writeln!(std::io::stderr().lock(), "Error: accept failed: {}", e);
                    }
                }
            }
        }
    }

    /// Checks if a filename contains unsafe path characters
    fn is_safe_filename(filename: &str) -> bool {
        !filename.contains('/')
            && !filename.contains('\\')
            && !filename.contains('$')
    }

    /// Handles a single client connection
    /// - Reads a token from the connection
    /// - If "quit", sets CANCEL_FLAG and returns
    /// - Otherwise, treats token as filename and streams file contents
    fn handle_connection(mut stream: TcpStream) {
        let mut reader = BufReader::new(&stream);
        let mut token = String::new();
        
        // Failed to read from connection
        if reader.read_line(&mut token).is_err() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
        

        let token = token.trim();        
        if token == "quit" {
            CANCEL_FLAG.store(true, Ordering::SeqCst);
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
        
        // ensure filename doesn't contain path traversal characters
        if !Self::is_safe_filename(token) {
            let _ = writeln!(std::io::stderr().lock(), "Warning: Bad filename requested: '{}'", token);
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
        
        let file = match File::open(token) {
            Ok(f) => f,
            Err(e) => {
                let _ = writeln!(std::io::stderr().lock(), "Error: Failed to open file '{}': {}", token, e);
                let _ = stream.shutdown(std::net::Shutdown::Both);
                return;
            }
        };
        
        // Open file -> read file to buffer
        let mut file_reader = BufReader::new(file);
        let mut buffer = Vec::new();
        if let Err(e) = file_reader.read_to_end(&mut buffer) {
            let _ = writeln!(std::io::stderr().lock(), "Error: Failed to read file '{}': {}", token, e);
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
        if let Err(e) = stream.write_all(&buffer) {
            let _ = writeln!(std::io::stderr().lock(), "Error: Failed to write to connection: {}", e);
        }
        
        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
}