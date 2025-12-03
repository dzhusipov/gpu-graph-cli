use chrono::{TimeDelta, Utc};
use std::process::Command;

use crate::models::{GpuDataPoint, GpuInfo};

/// Fetches GPU data from nvidia-smi and updates the provided GPU list.
/// Returns the number of data points kept (last 60 minutes).
pub fn fetch_gpu_data(gpus: &mut Vec<GpuInfo>) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=index,name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw")
        .arg("--format=csv,noheader,nounits")
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let now = Utc::now();

    for (idx, line) in output_str.lines().enumerate() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 7 {
            let gpu_index: usize = parts[0].parse().unwrap_or(idx);
            let name = parts[1].to_string();
            let gpu_util: f64 = parts[2].parse().unwrap_or(0.0);
            let memory_used: f64 = parts[3].parse().unwrap_or(0.0);
            let memory_total: f64 = parts[4].parse().unwrap_or(0.0);
            let temperature: f64 = parts[5].parse().unwrap_or(0.0);
            let power_usage: f64 = parts[6].parse().unwrap_or(0.0);

            let data_point = GpuDataPoint {
                timestamp: now,
                gpu_util,
                memory_used,
                memory_total,
                temperature,
                power_usage,
            };

            // Ensure GPU vector has enough elements
            while gpus.len() <= gpu_index {
                gpus.push(GpuInfo::new(format!("GPU {}", gpus.len())));
            }

            gpus[gpu_index].name = name;
            gpus[gpu_index].data_points.push_back(data_point);

            // Keep last 60 minutes of data
            let cutoff = now - TimeDelta::try_minutes(60).unwrap_or_default();
            while let Some(front) = gpus[gpu_index].data_points.front() {
                if front.timestamp < cutoff {
                    gpus[gpu_index].data_points.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}
