use chrono::{TimeDelta, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

use crate::models::{GpuDataPoint, GpuInfo};
use crate::theme::*;

/// Renders the main UI with header and GPU panels
pub fn render(frame: &mut Frame, gpus: &[GpuInfo], frame_count: u64) {
    // Main container with dark background
    let main_block = Block::default().style(Style::default().bg(DARK_BG));
    frame.render_widget(main_block, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(0),    // GPU panels
        ])
        .split(frame.area());

    render_header(frame, chunks[0], gpus, frame_count);

    if gpus.is_empty() {
        render_no_gpu(frame, chunks[1], frame_count);
        return;
    }

    // Dynamic layout for GPUs
    let gpu_count = gpus.len();
    let gpu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            (0..gpu_count)
                .map(|_| Constraint::Ratio(1, gpu_count as u32))
                .collect::<Vec<_>>(),
        )
        .split(chunks[1]);

    for (gpu_idx, gpu) in gpus.iter().enumerate() {
        render_gpu(frame, gpu_idx, gpu, gpu_chunks[gpu_idx]);
    }
}

fn render_header(frame: &mut Frame, area: Rect, gpus: &[GpuInfo], frame_count: u64) {
    let now = Utc::now();
    let uptime = gpus
        .first()
        .and_then(|g| g.data_points.front())
        .map(|dp| now - dp.timestamp)
        .unwrap_or(TimeDelta::zero());

    let glitch_char = if frame_count % 10 < 2 { "█" } else { " " };

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
                format!("GPUs: {}", gpus.len()),
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
    frame.render_widget(header, area);
}

fn render_no_gpu(frame: &mut Frame, area: Rect, frame_count: u64) {
    let blink = if frame_count % 20 < 10 { "█" } else { " " };
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
    frame.render_widget(paragraph, area);
}

fn render_gpu(frame: &mut Frame, gpu_idx: usize, gpu: &GpuInfo, area: Rect) {
    let data: Vec<&GpuDataPoint> = gpu.data_points.iter().collect();

    if data.is_empty() {
        return;
    }

    let latest = data.last().unwrap();

    // Status color based on load
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

    // GPU block
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
            Span::styled(&gpu.name, Style::default().fg(CYBER_BLUE)),
            Span::styled(" ", Style::default()),
        ])
        .style(Style::default().bg(DARK_BG));

    frame.render_widget(gpu_block, area);

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
    frame.render_widget(stats, inner[0]);

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
    frame.render_widget(util_gauge, inner[1]);

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
    frame.render_widget(mem_gauge, inner[2]);

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
    frame.render_widget(util_sparkline, spark_chunks[0]);

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
    frame.render_widget(temp_sparkline, spark_chunks[1]);
}
