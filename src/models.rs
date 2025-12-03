use chrono::{DateTime, Utc};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct GpuDataPoint {
    pub timestamp: DateTime<Utc>,
    pub gpu_util: f64,
    pub memory_used: f64,
    pub memory_total: f64,
    pub temperature: f64,
    pub power_usage: f64,
}

#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub name: String,
    pub data_points: VecDeque<GpuDataPoint>,
}

impl GpuInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            data_points: VecDeque::new(),
        }
    }
}
