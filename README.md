# ntomb – undead connections monitor

**A terminal TUI that visualizes network "undead" connections using graveyard and coffin metaphors**

> **Kiroween 2025 Submission Version**: [`kiroween-2025-submission`](https://github.com/yourusername/ntomb/releases/tag/kiroween-2025-submission)  
> This tag marks the official submission version for the Kiroween 2025 hackathon (submitted December 5, 2025).  
> Development continues on the main branch with bug fixes and new features.

---

## Introduction

ntomb is a terminal-based monitoring tool that visualizes network connections on Linux systems in real-time. Unlike traditional tools like `netstat` and `ss` that display connections as flat lists, ntomb presents the relationship between hosts and endpoints intuitively through a **Halloween-themed graphical interface**.

Network endpoints are arranged radially around a central coffin (⚰️), with different icons and colors representing connection states. While leveraging "undead" metaphors like zombie processes (💀), active connections (🎃), and fading connections (👻), ntomb maintains the **clarity and readability** needed by SREs and security engineers in production environments.

---

## Features

### 🕸️ The Graveyard (Network Topology)
- **Central HOST Coffin (⚰️)**: Displays current host or selected process at the center
- **Radial Node Layout**: Endpoints arranged in 3 rings based on network zones (local/private/public)
- **Braille Art Rendering**: Smooth curves using Canvas widget with Braille markers
- **Connection State Visualization**: 
  - 🎃 ESTABLISHED (active connections)
  - 👻 TIME_WAIT (fading connections)
  - 💀 CLOSE_WAIT (zombie connections)
  - 👂 LISTEN (listening sockets)
- **Legend Display**: Icon meanings shown as `[⚰️ host 🏠 local 🎃 ext 👑 hot]`
- **Summary Statistics**: Real-time display of Endpoints, Listening, and Total counts

### 🔮 Soul Inspector (Detail Panel)
- **Target Information**: Detailed info for selected host/process
  - TARGET: Name and icon
  - ROLE: Server/client/public connection counts
  - STATE: Active/listening connection status
  - CONN: Total connection count and PID
  - RISK: Suspicious connection detection (high-port, non-standard patterns)
  - BPF: Refresh interval display
- **Blockified Layout**: Information clearly organized by category

### 📊 Traffic History (Last 60s)
- **Real-time Activity Sparkline**: Visualizes network activity over the last 60 seconds
- **Statistics Display**: Shows Avg/Peak activity scores
- **Mode-specific Data**: Different data for Host mode (all connections) vs Process mode (selected process)

### 📜 Open Sockets / 🌐 Active Connections
- **Connection List**: All active connections in a scrollable list
- **Process Information**: Owning process shown with `[name(pid)]` tag
- **State-based Colors**: ESTABLISHED (green), LISTEN (white), TIME_WAIT (orange), CLOSE (red)
- **Selection Highlight**: Currently selected connection highlighted with Deep Indigo background

### 🎨 Kiroween Overdrive Mode
- **Enhanced Halloween Theme**: Toggleable enhanced visual effects with 'H' key
- **Animations**: Dynamic visual effects like pulse and zombie blinking (toggle with 'A' key)
- **Adaptive Performance**: Automatically reduces animation complexity when connection count is high

### ⌨️ Keyboard Navigation
- **Intuitive Shortcuts**: Always displayed in the status bar at the bottom
- **Mode Switching**: Toggle between Host mode ↔ Process mode with 'P' key
- **Refresh Rate Control**: Real-time adjustment with '+'/'-' keys
- **Panel Switching**: Move focus with Tab key

### 🔧 .kiro-based Design
- **Spec-driven Development**: Requirements, design, and tasks documented in `.kiro/specs/`
- **Steering Guides**: Visual design, security domain, and coding style guides in `.kiro/steering/`
- **MCP Integration**: Model Context Protocol server implementation in `ntomb_mcp/` (suspicious detection rules)

---

## Screenshots

<!-- TODO: add main UI screenshot (Graveyard + Soul Inspector + Traffic History) -->

<!-- TODO: add Host mode vs Process mode comparison -->

<!-- TODO: add Kiroween Overdrive mode demo -->

<!-- TODO: add suspicious connections detection demo -->

---

## Installation

### Requirements
- **OS**: Linux (macOS has limited support)
- **Rust**: 1.70 or higher
- **Dependencies**: 
  - `netstat2` (cross-platform socket information)
  - `sysinfo` (process information)
  - `ratatui` + `crossterm` (TUI rendering)

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/ntomb
cd ntomb

# Build
cargo build --release

# Run
./target/release/ntomb
```

### Install via Cargo

```bash
cargo install --path .
```

---

## Usage

### Basic Execution

```bash
# Run in Host mode (default)
ntomb

# Focus on specific process (switch with 'P' key after launch)
ntomb
# → Select a connection and press 'P' key
```

### Common Use Cases

1. **Finding Undead Connections on Local Development Machine**
   - Run in Host mode to check TIME_WAIT and CLOSE_WAIT connections across the system
   - Discover zombie processes or resource leak patterns

2. **Monitoring Network Activity of Specific Process**
   - Select a suspicious connection and press 'P' key to focus on that process
   - Analyze activity patterns over the last 60 seconds using Traffic History

3. **Detecting Security Anomalies**
   - Check RISK line for suspicious connections (high-port, non-standard patterns)
   - Discover unexpected connections to public IPs

4. **Network Debugging**
   - Real-time monitoring of connection states between services
   - Identify performance issues using latency-based ring layout

---

## Interaction / Keybindings

| Key | Description |
|-----|-------------|
| `↑` / `↓` | Move up/down in connection list |
| `Tab` | Switch panel (Graveyard ↔ Soul Inspector ↔ Grimoire) |
| `P` | Toggle process focus (Host ↔ Process mode) |
| `+` / `=` | Decrease refresh rate (increase interval) |
| `-` / `_` | Increase refresh rate (decrease interval) |
| `A` | Toggle animations (pulse, zombie blinking, etc.) |
| `H` | Toggle Kiroween Overdrive mode (enhanced Halloween theme) |
| `T` | Toggle endpoint labels (show/hide IP:port) |
| `Q` / `Esc` | Quit |

**Status Bar Indicators:**
- `[A:ON/OFF]` - Animation state
- `[H:ON/OFF]` - Overdrive mode state
- `[t:ON/OFF]` - Label display state

---

## Architecture / Design

### Core Components

- **`src/net/mod.rs`**: Network connection scanning
  - Cross-platform socket information collection using `netstat2` library
  - TCP connection state parsing and Connection struct creation

- **`src/procfs/mod.rs`**: Process mapping (Linux-only)
  - Socket inode extraction by scanning `/proc/<pid>/fd/*`
  - Process name reading from `/proc/<pid>/comm`
  - Graceful handling of permission errors

- **`src/app/mod.rs`**: Application state management
  - `AppState`: Connection data, mode, settings, animation state
  - `GraveyardMode`: Host / Process mode switching
  - `RefreshConfig`: Dynamic refresh interval adjustment

### UI Layer

- **`src/ui/banner.rs`**: Header (title, tagline, global statistics)
- **`src/ui/graveyard.rs`**: Network topology map
  - Canvas widget + Braille markers
  - Network zone-based ring layout (local/private/public)
  - Coffin rendering and exclusion zone
- **`src/ui/inspector.rs`**: Soul Inspector + Traffic History
  - Blockified information layout
  - Activity history display using Sparkline widget
- **`src/ui/grimoire.rs`**: Connection list (Open Sockets / Active Connections)
- **`src/ui/status_bar.rs`**: Bottom status bar (key bindings, toggle states)

### Theme System

- **`src/theme/mod.rs`**: Color palette definition
  - Neon Purple, Pumpkin Orange, Blood Red, Toxic Green, Bone White
  - Icon mapping for Overdrive mode

### .kiro Spec Structure

```
.kiro/
├── specs/
│   ├── ui-skeleton/          # UI layout and interaction
│   ├── process-focus/        # Process focus feature
│   ├── configurable-refresh/ # Refresh rate control
│   ├── graveyard-adaptive-layout/ # Adaptive layout
│   ├── ntomb-graveyard-vfx/  # Visual effects and animations
│   ├── network_map.yaml      # Network map configuration
│   └── suspicious_detection.yaml # Suspicious connection detection rules
└── steering/
    ├── visual-design.md      # Color, layout, widget design guide
    ├── security-domain.md    # Security principles, read-only, detection heuristics
    └── ntomb-coding-style.md # Rust coding style, testing strategy
```

---

## Limitations / Roadmap

### Current Limitations

- **Linux Primary Support**: macOS has limited support (no procfs functionality)
- **Root Privileges**: sudo required to view process information of other users
- **Terminal Size**: Minimum 80x24 recommended; smaller sizes may break layout
- **Actual Byte Transfer**: Currently displays connection activity score only (kB/s not supported)
- **BPF Integration**: eBPF-based real-time packet capture not yet implemented

### Planned Features

- [ ] **Actual Byte Transfer Display**: `ss -i` parsing or eBPF integration
- [ ] **Enhanced Suspicious Detection**: Expand `.kiro/specs/suspicious_detection.yaml` rules
- [ ] **Full MCP Server Integration**: External tool integration via ntomb_mcp
- [ ] **Filtering and Search**: Filter by specific IP, port, or process name
- [ ] **Log Export**: Save connection history to JSON/CSV
- [ ] **Plugin System**: Custom detection rules and visualization extensions

---

## Development

### Development Environment Setup

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Code formatting
cargo fmt

# Linting
cargo clippy

# Release build (optimized + stripped)
cargo build --release
```

### Testing Strategy

- **Unit Tests**: Located in `#[cfg(test)]` blocks within each module
- **Property-Based Tests**: Using `proptest` (some planned for implementation)
- **Integration Tests**: In `tests/` directory (to be added)

### Code Structure Principles

- **Read-only Principle**: Never modifies system state (security-domain.md)
- **Graceful Degradation**: Elegantly handles permission errors, platform differences, etc.
- **Clear Separation**: Distinct layers for data collection (net, procfs) / business logic (app) / UI (ui)

---

## Contributing

ntomb is an open-source project and welcomes contributions!

### Contribution Guidelines

1. **Code Style**: Must pass `cargo fmt` and `cargo clippy`
2. **Testing**: Add tests for new features
3. **Documentation**: Write doc comments for public APIs
4. **Issues/PRs**: Use GitHub Issues and Pull Requests

Bug reports, feature suggestions, and code contributions are all welcome!

---

## License

MIT License

See [LICENSE](LICENSE) file for details.

---

## Credits

**Built with:**
- [Ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal control
- [netstat2](https://github.com/zhongzc/netstat2) - Network socket information library
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System/process information
- [Kiro AI](https://kiro.ai) - Spec-driven development assistant

**Inspired by:**
- `netstat`, `ss`, `lsof`, `iftop` - Classic network tools
- Halloween 🎃 - Inspiration for the undead metaphor

---

**💀 "Revealing the unseen connections of the undead." 💀**
