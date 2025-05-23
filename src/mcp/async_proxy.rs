use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout, ChildStderr};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use anyhow::{Result, Context};

use crate::mcp::buffer::MCPBuffer;
use crate::output::is_quiet_mode;

pub struct AsyncStdioProxy {
    buffer: Arc<MCPBuffer>,
    container: Child,
    shutdown_tx: mpsc::Sender<()>,
}

impl AsyncStdioProxy {
    pub fn new(buffer: Arc<MCPBuffer>, container: Child) -> Result<Self> {
        let (shutdown_tx, _) = mpsc::channel(1);
        
        Ok(Self {
            buffer,
            container,
            shutdown_tx,
        })
    }

    pub async fn start(mut self) -> Result<()> {
        let buffer = self.buffer.clone();
        let _shutdown_tx = self.shutdown_tx.clone();
        
        let container_stdin = self.container.stdin.take()
            .context("Failed to capture container stdin")?;
        let container_stdout = self.container.stdout.take()
            .context("Failed to capture container stdout")?;
        let container_stderr = self.container.stderr.take()
            .context("Failed to capture container stderr")?;
        
        // Create shutdown receiver for each task
        let (shutdown_tx_stdin, mut shutdown_rx_stdin) = mpsc::channel::<()>(1);
        let (shutdown_tx_stdout, mut shutdown_rx_stdout) = mpsc::channel::<()>(1);
        let (shutdown_tx_stderr, mut shutdown_rx_stderr) = mpsc::channel::<()>(1);
        let (shutdown_tx_timeout, mut shutdown_rx_timeout) = mpsc::channel::<()>(1);
        
        // Spawn stdin handler (client -> server)
        let stdin_task = {
            let buffer = buffer.clone();
            tokio::spawn(async move {
                handle_stdin(buffer, container_stdin, &mut shutdown_rx_stdin).await
            })
        };

        // Spawn stdout handler (server -> client)
        let stdout_task = {
            let buffer = buffer.clone();
            tokio::spawn(async move {
                handle_stdout(buffer, container_stdout, &mut shutdown_rx_stdout).await
            })
        };

        // Spawn stderr handler
        let stderr_task = {
            tokio::spawn(async move {
                handle_stderr(container_stderr, &mut shutdown_rx_stderr).await
            })
        };

        // Spawn timeout checker
        let timeout_task = {
            let buffer = buffer.clone();
            tokio::spawn(async move {
                handle_timeout(buffer, &mut shutdown_rx_timeout).await
            })
        };

        // Wait for container to finish
        let exit_status = self.container.wait().await?;
        
        // Signal all tasks to shutdown
        let _ = shutdown_tx_stdin.send(()).await;
        let _ = shutdown_tx_stdout.send(()).await;
        let _ = shutdown_tx_stderr.send(()).await;
        let _ = shutdown_tx_timeout.send(()).await;
        
        // Wait for all tasks to complete
        let _ = tokio::join!(stdin_task, stdout_task, stderr_task, timeout_task);
        
        if exit_status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Container exited with non-zero status: {}", exit_status))
        }
    }
}

async fn handle_stdin(
    buffer: Arc<MCPBuffer>,
    mut container_stdin: ChildStdin,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    let mut stdin = tokio::io::stdin();
    let mut read_buffer = vec![0u8; 8192];
    
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => break,
            result = stdin.read(&mut read_buffer) => {
                match result {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = read_buffer[..n].to_vec();
                        
                        if buffer.is_server_ready() {
                            // Server is ready, write directly
                            if container_stdin.write_all(&data).await.is_err() {
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
                        if container_stdin.write_all(&message).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn handle_stdout(
    buffer: Arc<MCPBuffer>,
    mut container_stdout: ChildStdout,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    let mut stdout = tokio::io::stdout();
    let mut read_buffer = vec![0u8; 8192];
    
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => break,
            result = container_stdout.read(&mut read_buffer) => {
                match result {
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
                        if stdout.write_all(&data).await.is_err() {
                            break;
                        }
                        let _ = stdout.flush().await;
                    }
                    Err(_) => break,
                }
            }
        }
    }
    
    // Flush any remaining server messages
    for message in buffer.drain_server_buffer() {
        let _ = stdout.write_all(&message).await;
    }
    let _ = stdout.flush().await;
    
    Ok(())
}

async fn handle_stderr(
    mut container_stderr: ChildStderr,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    let mut stderr = tokio::io::stderr();
    let mut read_buffer = vec![0u8; 8192];
    
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => break,
            result = container_stderr.read(&mut read_buffer) => {
                match result {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // Write to client stderr immediately
                        if stderr.write_all(&read_buffer[..n]).await.is_err() {
                            break;
                        }
                        let _ = stderr.flush().await;
                    }
                    Err(_) => break,
                }
            }
        }
    }
    
    Ok(())
}

async fn handle_timeout(
    buffer: Arc<MCPBuffer>,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    let mut check_interval = interval(Duration::from_millis(100));
    
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => break,
            _ = check_interval.tick() => {
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
            }
        }
    }
    
    Ok(())
}