use chrono::{DateTime, Duration, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
    Frame, Terminal,
};
use std::collections::VecDeque;
use std::io;
use std::process::Command;
use std::time::Instant;

// Приглушённая хакерская цветовая схема
const NEON_GREEN: Color = Color::Rgb(0, 160, 50);
const NEON_CYAN: Color = Color::Rgb(0, 150, 160);
const NEON_MAGENTA: Color = Color::Rgb(160, 60, 160);
const NEON_YELLOW: Color = Color::Rgb(180, 160, 60);
const NEON_RED: Color = Color::Rgb(180, 60, 60);
const DARK_BG: Color = Color::Rgb(15, 15, 25);
const MATRIX_GREEN: Color = Color::Rgb(30, 130, 30);
const CYBER_BLUE: Color = Color::Rgb(60, 130, 180);

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
    name: String,
    data_points: VecDeque<GpuDataPoint>,
}

struct App {
    gpus: Vec<GpuInfo>,
    last_update: Instant,
    frame_count: u64,
}

impl App {
    fn new() -> Self {
        App {
            gpus: Vec::new(),
            last_update: Instant::now(),
            frame_count: 0,
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

                while self.gpus.len() <= gpu_index {
                    self.gpus.push(GpuInfo {
                        name: format!("GPU {}", self.gpus.len()),
                        data_points: VecDeque::new(),
                    });
                }

                self.gpus[gpu_index].name = name;
                self.gpus[gpu_index].data_points.push_back(data_point);

                // Keep last 60 minutes of data
                let cutoff = now - Duration::minutes(60);
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

    fn get_data(&self, gpu_idx: usize) -> Vec<GpuDataPoint> {
        if gpu_idx >= self.gpus.len() {
            return Vec::new();
        }
        self.gpus[gpu_idx].data_points.iter().cloned().collect()
    }

    fn render(&mut self, f: &mut Frame) {
        self.frame_count += 1;

        // Основной контейнер с темным фоном
        let main_block = Block::default().style(Style::default().bg(DARK_BG));
        f.render_widget(main_block, f.size());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(5), // Header
                Constraint::Min(0),    // GPU panels
            ])
            .split(f.size());

        self.render_header(f, chunks[0]);

        if self.gpus.is_empty() {
            self.render_no_gpu(f, chunks[1]);
            return;
        }

        // Динамическое распределение места для GPU
        let gpu_count = self.gpus.len();
        let gpu_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..gpu_count)
                    .map(|_| Constraint::Ratio(1, gpu_count as u32))
                    .collect::<Vec<_>>(),
            )
            .split(chunks[1]);

        for (gpu_idx, _) in self.gpus.iter().enumerate() {
            self.render_gpu(f, gpu_idx, gpu_chunks[gpu_idx]);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let now = Utc::now();
        let uptime = self
            .gpus
            .first()
            .and_then(|g| g.data_points.front())
            .map(|dp| now - dp.timestamp)
            .unwrap_or(Duration::zero());

        let glitch_char = if self.frame_count % 10 < 2 {
            "█"
        } else {
            " "
        };

        let header_text = vec![
            Line::from(vec![Span::styled(
                "╔══════════════════════════════════════════════════════════════╗",
                Style::default().fg(NEON_GREEN),
            )]),
            Line::from(vec![
                Span::styled("║  ", Style::default().fg(NEON_GREEN)),
                Span::styled(glitch_char, Style::default().fg(NEON_MAGENTA)),
                Span::styled(
                    " GPU MONITOR ",
                    Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD),
                ),
                Span::styled("v1.0 ", Style::default().fg(CYBER_BLUE)),
                Span::styled("│ ", Style::default().fg(NEON_GREEN)),
                Span::styled(
                    format!("{}", now.format("%H:%M:%S")),
                    Style::default().fg(NEON_YELLOW),
                ),
                Span::styled(" │ ", Style::default().fg(NEON_GREEN)),
                Span::styled(
                    format!(
                        "UPTIME: {:02}:{:02}:{:02}",
                        uptime.num_hours(),
                        uptime.num_minutes() % 60,
                        uptime.num_seconds() % 60
                    ),
                    Style::default().fg(NEON_CYAN),
                ),
                Span::styled(" │ ", Style::default().fg(NEON_GREEN)),
                Span::styled(
                    format!("GPUs: {}", self.gpus.len()),
                    Style::default().fg(NEON_MAGENTA),
                ),
                Span::styled(format!("{:>2}║", ""), Style::default().fg(NEON_GREEN)),
            ]),
            Line::from(vec![Span::styled(
                "╚══════════════════════════════════════════════════════════════╝",
                Style::default().fg(NEON_GREEN),
            )]),
        ];

        let header = Paragraph::new(header_text)
            .style(Style::default().bg(DARK_BG))
            .alignment(Alignment::Left);
        f.render_widget(header, area);
    }

    fn render_no_gpu(&self, f: &mut Frame, area: Rect) {
        let blink = if self.frame_count % 20 < 10 {
            "█"
        } else {
            " "
        };
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(NEON_RED)),
                Span::styled(
                    "!",
                    Style::default()
                        .fg(NEON_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("] ", Style::default().fg(NEON_RED)),
                Span::styled(
                    "SCANNING FOR GPU DEVICES",
                    Style::default().fg(NEON_RED).add_modifier(Modifier::BOLD),
                ),
                Span::styled(blink, Style::default().fg(NEON_GREEN)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "      Waiting for nvidia-smi response...",
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(NEON_RED))
            .style(Style::default().bg(DARK_BG));

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    fn render_gpu(&self, f: &mut Frame, gpu_idx: usize, area: Rect) {
        let data = self.get_data(gpu_idx);

        if data.is_empty() {
            return;
        }

        let latest = data.last().unwrap();

        // Определяем цвет статуса по загрузке
        let status_color = if latest.gpu_util > 90.0 {
            NEON_RED
        } else if latest.gpu_util > 50.0 {
            NEON_YELLOW
        } else {
            NEON_GREEN
        };

        let temp_color = if latest.temperature > 80.0 {
            NEON_RED
        } else if latest.temperature > 60.0 {
            NEON_YELLOW
        } else {
            NEON_CYAN
        };

        // GPU блок
        let gpu_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(MATRIX_GREEN))
            .title(vec![
                Span::styled(" ◆ ", Style::default().fg(status_color)),
                Span::styled(
                    format!("GPU {} ", gpu_idx),
                    Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ ", Style::default().fg(MATRIX_GREEN)),
                Span::styled(&self.gpus[gpu_idx].name, Style::default().fg(CYBER_BLUE)),
                Span::styled(" ", Style::default()),
            ])
            .style(Style::default().bg(DARK_BG));

        f.render_widget(gpu_block, area);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Stats line
                Constraint::Length(3), // GPU Util bar
                Constraint::Length(3), // Memory bar
                Constraint::Min(3),    // Sparklines
            ])
            .split(area);

        // Stats line
        let stats_line = Line::from(vec![
            Span::styled("  ┌─ ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("UTIL: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:5.1}%", latest.gpu_util),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("TEMP: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:3.0}°C", latest.temperature),
                Style::default().fg(temp_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("PWR: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:6.1}W", latest.power_usage),
                Style::default().fg(NEON_YELLOW),
            ),
            Span::styled(" │ ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("MEM: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}/{:.0}MB", latest.memory_used, latest.memory_total),
                Style::default().fg(NEON_MAGENTA),
            ),
            Span::styled(" ─┐", Style::default().fg(MATRIX_GREEN)),
        ]);
        let stats = Paragraph::new(stats_line).style(Style::default().bg(DARK_BG));
        f.render_widget(stats, inner[0]);

        // GPU Utilization bar
        let util_label = format!("▓ GPU {:5.1}%", latest.gpu_util);
        let util_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(DARK_BG)),
            )
            .gauge_style(Style::default().fg(status_color).bg(Color::Rgb(20, 20, 30)))
            .percent(latest.gpu_util as u16)
            .label(Span::styled(
                util_label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        f.render_widget(util_gauge, inner[1]);

        // Memory bar
        let mem_percent = (latest.memory_used / latest.memory_total) * 100.0;
        let mem_label = format!("▓ MEM {:5.1}%", mem_percent);
        let mem_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(DARK_BG)),
            )
            .gauge_style(Style::default().fg(NEON_MAGENTA).bg(Color::Rgb(20, 20, 30)))
            .percent(mem_percent as u16)
            .label(Span::styled(
                mem_label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        f.render_widget(mem_gauge, inner[2]);

        // Sparklines row
        let spark_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner[3]);

        // GPU Util sparkline
        let util_data: Vec<u64> = data.iter().map(|dp| dp.gpu_util as u64).collect();
        let util_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(40, 80, 40)))
                    .title(Span::styled(" ◇ UTIL% ", Style::default().fg(NEON_GREEN)))
                    .style(Style::default().bg(DARK_BG)),
            )
            .data(&util_data)
            .style(Style::default().fg(MATRIX_GREEN))
            .max(100);
        f.render_widget(util_sparkline, spark_chunks[0]);

        // Temperature sparkline
        let temp_data: Vec<u64> = data.iter().map(|dp| dp.temperature as u64).collect();
        let temp_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(80, 40, 40)))
                    .title(Span::styled(" ◇ TEMP°C ", Style::default().fg(NEON_RED)))
                    .style(Style::default().bg(DARK_BG)),
            )
            .data(&temp_data)
            .style(Style::default().fg(NEON_RED))
            .max(100);
        f.render_widget(temp_sparkline, spark_chunks[1]);
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
    let _ = app.fetch_gpu_data();

    loop {
        // Update data every second
        if app.last_update.elapsed().as_secs() >= 1 {
            let _ = app.fetch_gpu_data();
            app.last_update = Instant::now();
        }

        terminal.draw(|f| app.render(f))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
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
