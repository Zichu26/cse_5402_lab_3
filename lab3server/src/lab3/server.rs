/// server.rs
/// Lab 3 - CSE 5402 Fall 2025
/// Author: Zichu Pan, Edgar Palomino
/// Summary: Multi-threaded file server implementation.
use std::io::Write;
use std::net::TcpListener;
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
}