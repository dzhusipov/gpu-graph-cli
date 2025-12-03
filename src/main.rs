use chrono::{DateTime, Duration, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame, Terminal,
};
use std::collections::VecDeque;
use std::io;
use std::process::Command;
use std::time::Instant;

#[derive(Clone, Debug)]
struct GpuDataPoint {
    timestamp: DateTime<Utc>,
    gpu_util: f64,
    memory_used: f64,
    memory_total: f64,
    temperature: f64,
    power_usage: f64,
}

#[derive(Clone, Debug)]
struct GpuInfo {
    index: usize,
    name: String,
    data_points: VecDeque<GpuDataPoint>,
}

struct App {
    gpus: Vec<GpuInfo>,
    selected_time_range: usize,
    time_ranges: Vec<(String, Duration)>,
    last_update: Instant,
    update_interval: Duration,
}

impl App {
    fn new() -> Self {
        let time_ranges = vec![
            ("1 hour".to_string(), Duration::hours(1)),
            ("30 minutes".to_string(), Duration::minutes(30)),
            ("15 minutes".to_string(), Duration::minutes(15)),
            ("10 minutes".to_string(), Duration::minutes(10)),
            ("5 minutes".to_string(), Duration::minutes(5)),
            ("3 minutes".to_string(), Duration::minutes(3)),
            ("1 minute".to_string(), Duration::minutes(1)),
            ("30 seconds".to_string(), Duration::seconds(30)),
            ("10 seconds".to_string(), Duration::seconds(10)),
        ];

        App {
            gpus: Vec::new(),
            selected_time_range: 0,
            time_ranges,
            last_update: Instant::now(),
            update_interval: Duration::seconds(1),
        }
    }

    fn fetch_gpu_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

                // Ensure we have enough GPU slots
                while self.gpus.len() <= gpu_index {
                    self.gpus.push(GpuInfo {
                        index: self.gpus.len(),
                        name: format!("GPU {}", self.gpus.len()),
                        data_points: VecDeque::new(),
                    });
                }

                self.gpus[gpu_index].name = name;
                self.gpus[gpu_index].data_points.push_back(data_point);

                // Keep only data within the longest time range (1 hour)
                let cutoff = now - Duration::hours(1);
                while let Some(front) = self.gpus[gpu_index].data_points.front() {
                    if front.timestamp < cutoff {
                        self.gpus[gpu_index].data_points.pop_front();
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_filtered_data(&self, gpu_idx: usize) -> Vec<GpuDataPoint> {
        if gpu_idx >= self.gpus.len() {
            return Vec::new();
        }

        let (_, duration) = &self.time_ranges[self.selected_time_range];
        let cutoff = Utc::now() - *duration;

        self.gpus[gpu_idx]
            .data_points
            .iter()
            .filter(|dp| dp.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Time range selector
                Constraint::Min(0),    // GPU graphs
            ])
            .split(f.size());

        // Time range selector
        let time_range_text: Vec<Span> = self
            .time_ranges
            .iter()
            .enumerate()
            .flat_map(|(i, (name, _))| {
                let style = if i == self.selected_time_range {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::Gray)
                };
                vec![Span::styled(format!("[{}]", name), style), Span::raw(" ")]
            })
            .collect();

        let time_range_block = Block::default()
            .borders(Borders::ALL)
            .title("Time Range (← → to change)");
        let time_range_paragraph =
            Paragraph::new(Line::from(time_range_text)).block(time_range_block);
        f.render_widget(time_range_paragraph, chunks[0]);

        // GPU graphs
        if self.gpus.is_empty() {
            let no_gpu_text = Paragraph::new("No GPU data available. Waiting for nvidia-smi...")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(no_gpu_text, chunks[1]);
            return;
        }

        let gpu_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..self.gpus.len())
                    .map(|_| Constraint::Length(15))
                    .collect::<Vec<_>>(),
            )
            .split(chunks[1]);

        for (gpu_idx, _gpu) in self.gpus.iter().enumerate() {
            self.render_gpu(f, gpu_idx, gpu_chunks[gpu_idx]);
        }
    }

    fn render_gpu(&self, f: &mut Frame, gpu_idx: usize, area: Rect) {
        let data = self.get_filtered_data(gpu_idx);

        if data.is_empty() {
            let no_data_text = Paragraph::new(format!(
                "GPU {}: {} - No data for selected time range",
                gpu_idx, self.gpus[gpu_idx].name
            ))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(no_data_text, area);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title and current stats
                Constraint::Length(3), // GPU Utilization sparkline
                Constraint::Length(3), // Memory Usage gauge
                Constraint::Length(3), // Temperature and Power
            ])
            .split(area);

        // Title and current stats
        let latest = data.last().unwrap();
        let title_text = format!(
            "GPU {}: {} | Util: {:.1}% | Mem: {:.0}MB/{:.0}MB ({:.1}%) | Temp: {:.0}°C | Power: {:.1}W",
            gpu_idx,
            self.gpus[gpu_idx].name,
            latest.gpu_util,
            latest.memory_used,
            latest.memory_total,
            (latest.memory_used / latest.memory_total) * 100.0,
            latest.temperature,
            latest.power_usage
        );
        let title_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("GPU {}", gpu_idx));
        let title_paragraph = Paragraph::new(title_text).block(title_block);
        f.render_widget(title_paragraph, chunks[0]);

        // GPU Utilization sparkline
        let util_data: Vec<u64> = data.iter().map(|dp| dp.gpu_util as u64).collect();
        let util_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("GPU Utilization (%)"),
            )
            .data(&util_data)
            .style(Style::default().fg(Color::Green))
            .max(100);
        f.render_widget(util_sparkline, chunks[1]);

        // Memory Usage gauge
        let memory_percent = (latest.memory_used / latest.memory_total) * 100.0;
        let memory_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Memory Usage"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(memory_percent as u16)
            .label(format!(
                "{:.0}MB / {:.0}MB ({:.1}%)",
                latest.memory_used, latest.memory_total, memory_percent
            ));
        f.render_widget(memory_gauge, chunks[2]);

        // Temperature and Power sparklines
        let temp_data: Vec<u64> = data.iter().map(|dp| dp.temperature as u64).collect();
        let power_data: Vec<u64> = data.iter().map(|dp| dp.power_usage as u64).collect();

        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[3]);

        let temp_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Temperature (°C)"),
            )
            .data(&temp_data)
            .style(Style::default().fg(Color::Red))
            .max(100);
        f.render_widget(temp_sparkline, stats_chunks[0]);

        let power_sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Power (W)"))
            .data(&power_data)
            .style(Style::default().fg(Color::Yellow))
            .max(500);
        f.render_widget(power_sparkline, stats_chunks[1]);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let mut app = App::new();

    // Initial fetch
    app.fetch_gpu_data()?;

    loop {
        // Update data every second
        if app.last_update.elapsed().as_secs() >= 1 {
            if let Err(e) = app.fetch_gpu_data() {
                eprintln!("Error fetching GPU data: {}", e);
            }
            app.last_update = Instant::now();
        }

        terminal.draw(|f| app.render(f))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Left => {
                            if app.selected_time_range > 0 {
                                app.selected_time_range -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if app.selected_time_range < app.time_ranges.len() - 1 {
                                app.selected_time_range += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
