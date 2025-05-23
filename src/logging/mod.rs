use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use std::io::Write;

pub struct LogManager {
    log_dir: PathBuf,
}

impl LogManager {
    pub fn new() -> Result<Self> {
        let log_dir = Self::get_logs_directory()?;
        fs::create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create logs directory: {}", log_dir.display()))?;
        
        Ok(Self { log_dir })
    }

    fn get_logs_directory() -> Result<PathBuf> {
        // Use XDG_STATE_HOME if available, otherwise fall back to ~/.local/state
        let state_home = if let Ok(xdg_state) = env::var("XDG_STATE_HOME") {
            PathBuf::from(xdg_state)
        } else {
            let home = env::var("HOME")
                .with_context(|| "HOME environment variable not set")?;
            PathBuf::from(home).join(".local").join("state")
        };

        Ok(state_home.join("finch-mcp").join("logs"))
    }

    pub fn log_build_start(&self, operation_type: &str, identifier: &str) -> Result<String> {
        let timestamp = Utc::now();
        let log_filename = format!("{}_{}_build_{}.log", 
            operation_type,
            Self::sanitize_identifier(identifier),
            timestamp.format("%Y%m%d_%H%M%S")
        );
        
        let log_path = self.log_dir.join(&log_filename);
        let mut file = fs::File::create(&log_path)
            .with_context(|| format!("Failed to create log file: {}", log_path.display()))?;

        writeln!(file, "=== Build Log for {} ===", operation_type)?;
        writeln!(file, "Identifier: {}", identifier)?;
        writeln!(file, "Started: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(file, "=")?;
        writeln!(file)?;

        Ok(log_filename)
    }

    pub fn append_to_log(&self, log_filename: &str, content: &str) -> Result<()> {
        let log_path = self.log_dir.join(log_filename);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open log file: {}", log_path.display()))?;

        writeln!(file, "{}", content)?;
        Ok(())
    }

    pub fn finish_build_log(&self, log_filename: &str, success: bool, duration_secs: u64) -> Result<()> {
        let timestamp = Utc::now();
        let status = if success { "SUCCESS" } else { "FAILED" };
        
        let content = format!(
            "\n=== Build {} ===\nCompleted: {}\nDuration: {}s\n",
            status,
            timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            duration_secs
        );

        self.append_to_log(log_filename, &content)
    }

    pub fn list_recent_logs(&self, limit: usize) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();
        
        if !self.log_dir.exists() {
            return Ok(entries);
        }

        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                if let Some(log_entry) = LogEntry::from_path(&path)? {
                    entries.push(log_entry);
                }
            }
        }

        // Sort by creation time, most recent first
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        entries.truncate(limit);

        Ok(entries)
    }

    pub fn cleanup_old_logs(&self, keep_days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(keep_days as i64);
        let mut removed_count = 0;

        if !self.log_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                let metadata = fs::metadata(&path)?;
                if let Ok(created) = metadata.created() {
                    let created_datetime: DateTime<Utc> = created.into();
                    if created_datetime < cutoff {
                        fs::remove_file(&path)?;
                        removed_count += 1;
                    }
                }
            }
        }

        Ok(removed_count)
    }

    pub fn get_logs_directory_path(&self) -> &Path {
        &self.log_dir
    }

    fn sanitize_identifier(identifier: &str) -> String {
        identifier
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect()
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub filename: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub operation_type: String,
    pub identifier: String,
}

impl LogEntry {
    fn from_path(path: &Path) -> Result<Option<Self>> {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

        // Parse filename: {operation_type}_{identifier}_build_{timestamp}.log
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() < 4 || !filename.ends_with(".log") {
            return Ok(None);
        }

        let operation_type = parts[0].to_string();
        
        // Find the timestamp part (last part before .log)
        let _timestamp_part = parts.iter()
            .rev()
            .find(|&&part| part.contains(".log"))
            .map(|&part| part.trim_end_matches(".log"))
            .ok_or_else(|| anyhow::anyhow!("No timestamp found"))?;

        // Identifier is everything between operation_type and build_{timestamp}
        let identifier_parts = &parts[1..parts.len()-2]; // Skip operation_type and build_{timestamp}
        let identifier = identifier_parts.join("_");

        let metadata = fs::metadata(path)?;
        let created_at: DateTime<Utc> = metadata.created()?.into();

        Ok(Some(Self {
            filename: filename.to_string(),
            path: path.to_path_buf(),
            created_at,
            operation_type,
            identifier,
        }))
    }
}