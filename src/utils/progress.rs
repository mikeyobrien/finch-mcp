use std::io;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub struct BuildProgress {
    pb: ProgressBar,
    start_time: Instant,
    current_step: Arc<Mutex<String>>,
}

impl BuildProgress {
    pub fn new(total_steps: u64) -> Self {
        let pb = ProgressBar::new(total_steps);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}")
                .unwrap()
                .progress_chars("##-")
        );
        
        let start_time = Instant::now();
        let current_step = Arc::new(Mutex::new(String::new()));
        
        Self {
            pb,
            start_time,
            current_step,
        }
    }
    
    pub fn set_message(&self, msg: &str) {
        self.pb.set_message(msg.to_string());
        if let Ok(mut step) = self.current_step.lock() {
            *step = msg.to_string();
        }
    }
    
    pub fn inc(&self) {
        self.pb.inc(1);
    }
    
    pub fn finish_with_message(&self, msg: &str) {
        self.pb.finish_with_message(msg.to_string());
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

pub struct DockerBuildProgress {
    progress: BuildProgress,
    build_phases: Vec<&'static str>,
    current_phase: usize,
}

impl DockerBuildProgress {
    pub fn new() -> Self {
        let build_phases = vec![
            "Setting up build context",
            "Pulling base image", 
            "Installing dependencies",
            "Copying application files",
            "Finalizing image",
        ];
        
        let progress = BuildProgress::new(build_phases.len() as u64);
        
        Self {
            progress,
            build_phases,
            current_phase: 0,
        }
    }
    
    pub fn start_phase(&mut self, phase_index: usize) {
        if phase_index < self.build_phases.len() {
            self.current_phase = phase_index;
            let phase_name = self.build_phases[phase_index];
            self.progress.set_message(&format!("🔨 {}", phase_name));
            self.progress.inc();
        }
    }
    
    pub fn update_message(&self, msg: &str) {
        let phase_name = if self.current_phase < self.build_phases.len() {
            self.build_phases[self.current_phase]
        } else {
            "Building"
        };
        self.progress.set_message(&format!("🔨 {}: {}", phase_name, msg));
    }
    
    pub fn finish_success(&self, image_name: &str) {
        let elapsed = self.progress.elapsed();
        self.progress.finish_with_message(&format!(
            "✅ Built {} in {:.1}s", 
            style(image_name).green().bold(),
            elapsed.as_secs_f64()
        ));
    }
    
    pub fn finish_error(&self, error: &str) {
        self.progress.finish_with_message(&format!("❌ Build failed: {}", style(error).red()));
    }
}

pub fn run_build_with_progress(
    build_command: &mut Command,
    image_name: &str,
    project_type: &str,
) -> Result<()> {
    let mut progress = DockerBuildProgress::new();
    
    // Start the build process
    println!("\n{} Containerizing {} project...", 
        style("🚀").blue(), 
        style(project_type).cyan().bold()
    );
    
    progress.start_phase(0); // Setting up build context
    
    // Configure command to capture output
    let mut child = build_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start build process")?;
    
    // Handle stdout in a separate thread to parse Docker build output
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    // Parse Docker build output to track progress
    let progress_clone = Arc::new(Mutex::new(progress));
    let progress_thread = {
        let progress = progress_clone.clone();
        thread::spawn(move || {
            parse_docker_output(stdout, progress);
        })
    };
    
    // Handle stderr
    let error_thread = thread::spawn(move || {
        use std::io::Read;
        let mut error_output = String::new();
        io::BufReader::new(stderr).read_to_string(&mut error_output).ok();
        error_output
    });
    
    // Wait for the process to complete
    let exit_status = child.wait().context("Failed to wait for build process")?;
    
    // Wait for output parsing to complete
    progress_thread.join().unwrap();
    let error_output = error_thread.join().unwrap();
    
    // Finish progress based on result
    let progress = progress_clone.lock().unwrap();
    if exit_status.success() {
        progress.finish_success(image_name);
        println!("{} Container ready! Starting server...\n", style("✨").green());
    } else {
        let error_msg = if !error_output.trim().is_empty() {
            error_output.trim()
        } else {
            "Unknown build error"
        };
        progress.finish_error(error_msg);
        return Err(anyhow::anyhow!("Build failed with status: {}", exit_status));
    }
    
    Ok(())
}

fn parse_docker_output(
    stdout: std::process::ChildStdout,
    progress: Arc<Mutex<DockerBuildProgress>>,
) {
    use std::io::{BufRead, BufReader};
    
    let reader = BufReader::new(stdout);
    let mut current_phase = 0;
    
    for line in reader.lines() {
        if let Ok(line) = line {
            // Parse Docker build steps to track progress
            if line.contains("FROM ") && current_phase == 0 {
                if let Ok(mut p) = progress.lock() {
                    p.start_phase(1); // Pulling base image
                    current_phase = 1;
                }
            } else if (line.contains("RUN pip install") || 
                      line.contains("RUN npm install") || 
                      line.contains("RUN poetry install") ||
                      line.contains("RUN uv pip install")) && current_phase <= 1 {
                if let Ok(mut p) = progress.lock() {
                    p.start_phase(2); // Installing dependencies
                    current_phase = 2;
                }
            } else if line.contains("COPY . .") && current_phase <= 2 {
                if let Ok(mut p) = progress.lock() {
                    p.start_phase(3); // Copying application files
                    current_phase = 3;
                }
            } else if line.contains("exporting to image") && current_phase <= 3 {
                if let Ok(mut p) = progress.lock() {
                    p.start_phase(4); // Finalizing image
                    current_phase = 4;
                }
            }
            
            // Update with specific progress messages
            if line.contains("downloading") {
                if let Ok(p) = progress.lock() {
                    p.update_message("downloading base image");
                }
            } else if line.contains("extracting") {
                if let Ok(p) = progress.lock() {
                    p.update_message("extracting layers");
                }
            } else if line.contains("npm WARN") || line.contains("warning") {
                // Don't update for warnings, just continue
            } else if line.contains("found ") && line.contains("vulnerabilities") {
                if let Ok(p) = progress.lock() {
                    p.update_message("security scan complete");
                }
            }
        }
    }
}