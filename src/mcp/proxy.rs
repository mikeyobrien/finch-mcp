use std::io::{Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, ChildStderr};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use anyhow::{Result, Context};
use crossbeam_channel::{bounded, Sender, Receiver};

use crate::mcp::buffer::MCPBuffer;
use crate::output::is_quiet_mode;

pub struct StdioProxy {
    buffer: Arc<MCPBuffer>,
    container_stdin: Option<ChildStdin>,
    container_stdout: Option<ChildStdout>,
    container_stderr: Option<ChildStderr>,
    shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
}

impl StdioProxy {
    pub fn new(buffer: Arc<MCPBuffer>, mut container: Child) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = bounded(1);
        
        let container_stdin = container.stdin.take()
            .context("Failed to capture container stdin")?;
        let container_stdout = container.stdout.take()
            .context("Failed to capture container stdout")?;
        let container_stderr = container.stderr.take()
            .context("Failed to capture container stderr")?;
        
        Ok(Self {
            buffer,
            container_stdin: Some(container_stdin),
            container_stdout: Some(container_stdout),
            container_stderr: Some(container_stderr),
            shutdown_tx,
            shutdown_rx,
        })
    }

    pub fn start(mut self) -> Result<()> {
        let buffer = self.buffer.clone();
        let shutdown_rx = self.shutdown_rx.clone();
        
        // Spawn stdin handler (client -> server)
        let stdin_thread = {
            let buffer = buffer.clone();
            let mut container_stdin = self.container_stdin.take().unwrap();
            let shutdown_rx = shutdown_rx.clone();
            
            thread::spawn(move || {
                let mut client_stdin = std::io::stdin();
                let mut read_buffer = vec![0u8; 8192];
                
                loop {
                    // Check for shutdown
                    if shutdown_rx.try_recv().is_ok() {
                        break;
                    }
                    
                    // Read from client stdin
                    match client_stdin.read(&mut read_buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let data = read_buffer[..n].to_vec();
                            
                            if buffer.is_server_ready() {
                                // Server is ready, write directly
                                if container_stdin.write_all(&data).is_err() {
                                    break;
                                }
                            } else {
                                // Buffer the message
                                if buffer.buffer_client_message(data).is_err() {
                                    if !is_quiet_mode() {
                                        eprintln!("Warning: Client message buffer overflow");
                                    }
                                    break;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                    
                    // If server just became ready, flush buffered messages
                    if buffer.is_server_ready() {
                        for message in buffer.drain_client_buffer() {
                            if container_stdin.write_all(&message).is_err() {
                                break;
                            }
                        }
                    }
                }
            })
        };

        // Spawn stdout handler (server -> client)
        let stdout_thread = {
            let buffer = buffer.clone();
            let mut container_stdout = self.container_stdout.take().unwrap();
            let shutdown_rx = shutdown_rx.clone();
            
            thread::spawn(move || {
                let mut client_stdout = std::io::stdout();
                let mut read_buffer = vec![0u8; 8192];
                
                loop {
                    // Check for shutdown
                    if shutdown_rx.try_recv().is_ok() {
                        break;
                    }
                    
                    // Read from container stdout
                    match container_stdout.read(&mut read_buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let data = read_buffer[..n].to_vec();
                            
                            // Always check server messages for readiness
                            if let Err(e) = buffer.buffer_server_message(data.clone()) {
                                if !is_quiet_mode() {
                                    eprintln!("Warning: Server message buffer error: {}", e);
                                }
                            }
                            
                            // Write to client immediately
                            if client_stdout.write_all(&data).is_err() {
                                break;
                            }
                            let _ = client_stdout.flush();
                        }
                        Err(_) => break,
                    }
                }
                
                // Flush any remaining server messages
                for message in buffer.drain_server_buffer() {
                    let _ = client_stdout.write_all(&message);
                }
                let _ = client_stdout.flush();
            })
        };

        // Spawn stderr handler
        let stderr_thread = {
            let mut container_stderr = self.container_stderr.take().unwrap();
            let shutdown_rx = shutdown_rx.clone();
            
            thread::spawn(move || {
                let mut client_stderr = std::io::stderr();
                let mut read_buffer = vec![0u8; 8192];
                
                loop {
                    // Check for shutdown
                    if shutdown_rx.try_recv().is_ok() {
                        break;
                    }
                    
                    // Read from container stderr
                    match container_stderr.read(&mut read_buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            // Write to client stderr immediately
                            if client_stderr.write_all(&read_buffer[..n]).is_err() {
                                break;
                            }
                            let _ = client_stderr.flush();
                        }
                        Err(_) => break,
                    }
                }
            })
        };

        // Spawn timeout checker
        let timeout_thread = {
            let buffer = buffer.clone();
            let shutdown_rx = shutdown_rx.clone();
            
            thread::spawn(move || {
                loop {
                    // Check for shutdown
                    if shutdown_rx.try_recv().is_ok() {
                        break;
                    }
                    
                    // Check timeout
                    if let Err(e) = buffer.check_timeout() {
                        if !is_quiet_mode() {
                            eprintln!("Error: {}", e);
                        }
                        break;
                    }
                    
                    // If server is ready, no need to check timeout anymore
                    if buffer.is_server_ready() {
                        break;
                    }
                    
                    thread::sleep(Duration::from_millis(100));
                }
            })
        };

        // Wait for threads to complete
        let _ = stdin_thread.join();
        let _ = stdout_thread.join();
        let _ = stderr_thread.join();
        let _ = timeout_thread.join();
        
        Ok(())
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}