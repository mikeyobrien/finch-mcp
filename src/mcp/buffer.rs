use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct MCPBuffer {
    client_to_server: Arc<Mutex<VecDeque<Vec<u8>>>>,
    server_to_client: Arc<Mutex<VecDeque<Vec<u8>>>>,
    server_ready: Arc<AtomicBool>,
    startup_time: Arc<Mutex<Option<Instant>>>,
    max_buffer_size: usize,
    startup_timeout: Duration,
}

impl MCPBuffer {
    pub fn new(max_buffer_size: usize, startup_timeout: Duration) -> Self {
        Self {
            client_to_server: Arc::new(Mutex::new(VecDeque::new())),
            server_to_client: Arc::new(Mutex::new(VecDeque::new())),
            server_ready: Arc::new(AtomicBool::new(false)),
            startup_time: Arc::new(Mutex::new(Some(Instant::now()))),
            max_buffer_size,
            startup_timeout,
        }
    }

    pub fn buffer_client_message(&self, data: Vec<u8>) -> Result<()> {
        let mut buffer = self.client_to_server.lock().unwrap();
        
        // Check buffer size
        let current_size: usize = buffer.iter().map(|v| v.len()).sum();
        if current_size + data.len() > self.max_buffer_size {
            return Err(anyhow!("Buffer overflow: exceeds maximum size of {} bytes", self.max_buffer_size));
        }
        
        buffer.push_back(data);
        Ok(())
    }

    pub fn buffer_server_message(&self, data: Vec<u8>) -> Result<()> {
        // Check if this is the initialization response
        if !self.server_ready.load(Ordering::SeqCst) {
            if let Ok(text) = std::str::from_utf8(&data) {
                if text.contains(r#""method":"initialize""#) || 
                   text.contains(r#""result":{"capabilities""#) {
                    self.server_ready.store(true, Ordering::SeqCst);
                    *self.startup_time.lock().unwrap() = None;
                }
            }
        }
        
        let mut buffer = self.server_to_client.lock().unwrap();
        buffer.push_back(data);
        Ok(())
    }

    pub fn drain_client_buffer(&self) -> Vec<Vec<u8>> {
        let mut buffer = self.client_to_server.lock().unwrap();
        buffer.drain(..).collect()
    }

    pub fn drain_server_buffer(&self) -> Vec<Vec<u8>> {
        let mut buffer = self.server_to_client.lock().unwrap();
        buffer.drain(..).collect()
    }

    pub fn is_server_ready(&self) -> bool {
        self.server_ready.load(Ordering::SeqCst)
    }

    pub fn check_timeout(&self) -> Result<()> {
        if let Some(start_time) = *self.startup_time.lock().unwrap() {
            if start_time.elapsed() > self.startup_timeout {
                return Err(anyhow!("Server startup timeout after {:?}", self.startup_timeout));
            }
        }
        Ok(())
    }

    pub fn get_buffer_stats(&self) -> (usize, usize) {
        let client_size: usize = self.client_to_server.lock().unwrap().iter().map(|v| v.len()).sum();
        let server_size: usize = self.server_to_client.lock().unwrap().iter().map(|v| v.len()).sum();
        (client_size, server_size)
    }
}