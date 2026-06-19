# resource-monitor

Terminal resource monitor for Linux, built with [Rust](https://www.rust-lang.org/) and [Ratatui](https://ratatui.rs/).

## Features

- **5 tabs**: Processes, CPU, Memory, Network, Disk
- **Real-time charts**: CPU (per-core), RAM, Swap, Network IN/OUT, Disk I/O
- **Process table**: PID, User, Name, CPU%, Memory, Disk Read/Write — sortable, searchable, killable
- **CPU detail**: per-core usage, specs (brand, cores, cache, frequency, governor, temperature, load average, uptime)
- **Memory detail**: RAM & Swap usage charts + capacity info
- **Network detail**: IN/OUT charts + interface table (MAC, IPv4, IPv6)
- **Disk detail**: Read/Write charts + partition table (device, mount, FS, size, available, used%)
- **Alert**: panel border turns red when CPU > 90% or RAM > 85%
- **Export**: snapshot current processes to CSV (`~/resource-monitor-<timestamp>.csv`)
- **Search**: filter processes by name or PID
- **Catppuccin Macchiato** color theme with `--no-truecolor` fallback for older terminals

## Requirements

- Linux
- Rust 1.85+ (`cargo`)

## Build

```bash
cargo build --release
```

Binary will be at `./target/release/resource-monitor`.

## Usage

```bash
./target/release/resource-monitor [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--interval <ms>` | `500` | Refresh interval in milliseconds (min: 50) |
| `--no-mouse` | off | Disable mouse capture (recommended for some SSH sessions) |
| `--filter <query>` | — | Pre-fill process search filter on startup |
| `--no-truecolor` | off | Use 256-color ANSI fallback instead of TrueColor |
| `-h, --help` | — | Show help |

### Examples

```bash
# Faster refresh
./resource-monitor --interval 200

# Monitor specific process on startup
./resource-monitor --filter nginx

# SSH-friendly mode
./resource-monitor --no-mouse --no-truecolor
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `Tab` | Next tab |
| `1`–`5` | Jump to tab |
| `e` | Export process snapshot to CSV |
| `q` | Quit |

### Processes tab

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `/` | Search by name or PID |
| `s` | Sort menu (CPU, Memory, Disk Read, Disk Write) |
| `x` | Kill selected process (with confirmation) |
| `Ctrl+C` | Clear active filter |

### Search popup

| Key | Action |
|-----|--------|
| `Enter` / `Esc` | Close popup (filter stays active) |
| `Backspace` | Delete last character |
| `Ctrl+C` | Clear query and close |

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.30 | TUI framework |
| `crossterm` | 0.29 | Terminal backend |
| `sysinfo` | 0.39 | System information |
| `anyhow` | 1.0 | Error handling |
