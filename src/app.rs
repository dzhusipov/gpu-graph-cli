use std::time::Instant;

use crate::models::GpuInfo;
use crate::nvidia::fetch_gpu_data;

/// Main application state
pub struct App {
    pub gpus: Vec<GpuInfo>,
    pub last_update: Instant,
    pub frame_count: u64,
}

impl App {
    pub fn new() -> Self {
        App {
            gpus: Vec::new(),
            last_update: Instant::now(),
            frame_count: 0,
        }
    }

    /// Fetch GPU data from nvidia-smi
    pub fn update_gpu_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        fetch_gpu_data(&mut self.gpus)
    }

    /// Check if update is needed (every second)
    pub fn should_update(&self) -> bool {
        self.last_update.elapsed().as_secs() >= 1
    }

    /// Mark update as complete
    pub fn mark_updated(&mut self) {
        self.last_update = Instant::now();
    }

    /// Increment frame counter
    pub fn tick(&mut self) {
        self.frame_count += 1;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
