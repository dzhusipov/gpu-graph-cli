# GPU Graph CLI

A terminal-based utility written in Rust for monitoring NVIDIA GPU utilization via nvidia-smi with real-time graph visualization.

## Features

- Monitor multiple GPUs simultaneously
- Real-time display of GPU utilization, memory usage, temperature, and power consumption
- Cyberpunk/hacker-style visual theme with neon colors
- Data stored in memory only (no disk persistence)
- Data updates every second
- Stores last 60 minutes of metrics history

## Requirements

- Rust (version 1.75 or higher)
- NVIDIA drivers with nvidia-smi
- Linux (for nvidia-smi support)

## Dependencies

| Crate | Version | Description |
|-------|---------|-------------|
| ratatui | 0.29.0 | Terminal UI framework |
| crossterm | 0.29.0 | Cross-platform terminal manipulation |
| tokio | 1.48.0 | Async runtime |
| chrono | 0.4.42 | Date and time library |

## Project Structure

```
src/
├── main.rs      # Entry point, terminal setup/cleanup, main loop
├── app.rs       # App struct - application state management
├── models.rs    # GpuDataPoint, GpuInfo - data models
├── nvidia.rs    # fetch_gpu_data - nvidia-smi integration
├── theme.rs     # Color constants (NEON_GREEN, DARK_BG, etc.)
└── ui.rs        # UI rendering functions
```

### Architecture

- **`models.rs`** — Data structures (Data Layer)
- **`nvidia.rs`** — External system integration (Infrastructure Layer)
- **`theme.rs`** — Style configuration (Presentation Constants)
- **`ui.rs`** — Interface rendering (Presentation Layer)
- **`app.rs`** — Application business logic (Application Layer)
- **`main.rs`** — Initialization and main loop (Entry Point)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

Or after building:

```bash
./target/release/gpu-graph-cli
```

## Controls

- `q` or `Esc` - exit the program

## Docker

### Building the Image

```bash
docker build -t gpu-graph-cli .
```

### Running the Container

```bash
docker run --gpus all -it --rm gpu-graph-cli
```

Or use the convenience script:

```bash
./docker-run.sh
```

**Important**: nvidia-container-runtime is required for GPU access in Docker. Make sure it's installed on your system.

## Displayed Metrics

For each GPU, the following metrics are shown:
- **GPU Utilization** - percentage of GPU usage (progress bar + sparkline graph)
- **Memory Usage** - memory utilization (progress bar)
- **Temperature** - GPU temperature in °C (sparkline graph)
- **Power Usage** - power consumption in Watts

## Screenshot

The interface features a cyberpunk aesthetic with:
- Neon green, cyan, magenta, and yellow color scheme
- Real-time sparkline graphs
- Status indicators that change color based on load levels

## License

MIT
