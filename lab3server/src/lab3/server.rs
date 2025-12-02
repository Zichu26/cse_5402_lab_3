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
                            // Stuff
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
}