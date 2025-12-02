# ntomb

A Linux terminal app that visualizes network connections for a specific process with a subtle Halloween theme.

## Overview

ntomb is a process-centric network visualization tool that places a target process at the center and displays its network connections as a visual graph in the terminal. Unlike traditional tools like `netstat` or `ss` that show flat lists, ntomb makes it immediately clear which remote endpoints a process is communicating with.

**Target audience:** Linux SREs, security engineers, and backend developers debugging services.

**Kiroween theme:** Coffins for processes, pumpkins for active connections, ghosts for TIME_WAIT states, and skulls for suspicious endpoints - all while maintaining professional readability.

## Features

- **Process-centric visualization:** See all connections for a specific process
- **Interactive TUI:** Navigate connections with keyboard shortcuts (keyboard-only, no mouse)
- **Animated graphics:** Floating ghosts and smooth rendering with tview + tcell
- **Connection state awareness:** Visual indicators for ESTABLISHED, LISTEN, TIME_WAIT, etc.
- **Suspicious connection detection:** Flags unusual patterns (long-lived, high-port beaconing, etc.)
- **Dual themes:** Default (professional) and Halloween (spooky but readable)
- **Demo mode:** Run without system access for presentations and screenshots
- **Circular layout:** Process at center with endpoints arranged radially

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ntomb
cd ntomb

# Build
go build -o ntomb cmd/ntomb/main.go

# Or install
go install github.com/yourusername/ntomb/cmd/ntomb@latest
```

## Usage

```bash
# Demo mode (no system access required)
ntomb --demo

# Demo mode with Halloween theme
ntomb --demo --theme=halloween

# Inspect a specific process by PID (not yet implemented)
ntomb --pid 1234

# Inspect a process by name (not yet implemented)
ntomb --process nginx

# With verbose logging
ntomb --demo --verbose
```

## Keyboard Shortcuts

**Navigation:**
- `↑/k` - Move selection up
- `↓/j` - Move selection down

**Actions:**
- `r` - Refresh connection data
- `h` or `?` - Show help screen
- `q` or `ESC` - Quit application

**Note:** ntomb is keyboard-only. No mouse support.

## Project Status

**Current:** 
- ✅ Demo mode with realistic fake data
- ✅ tview + tcell based TUI with direct screen control
- ✅ Animated floating ghosts (Halloween theme)
- ✅ Circular network map layout
- ✅ Keyboard navigation (j/k, arrows)
- ✅ Connection details panel
- ✅ Dual themes (default + Halloween)

**TODO:**
- Implement real process scanning (`internal/process`)
- Implement real connection scanning (`internal/netscan`)
- Add connection state change detection
- Add auto-refresh functionality
- Add filtering and grouping logic

## Architecture

```
ntomb/
├── cmd/ntomb/          # CLI entry point
├── internal/
│   ├── graph/          # Graph model (nodes, edges, types)
│   ├── process/        # Process scanning (TODO)
│   ├── netscan/        # Connection scanning (TODO)
│   ├── theme/          # Visual themes (default, halloween)
│   ├── tui/            # Bubble Tea TUI (model, view, update)
│   └── demo/           # Demo mode with fake data
```

## Development

```bash
# Run in demo mode
go run cmd/ntomb/main.go --demo

# Run with Halloween theme
go run cmd/ntomb/main.go --demo --theme=halloween

# Run tests
go test ./...

# Format code
go fmt ./...

# Build release binary
go build -ldflags="-s -w" -o ntomb cmd/ntomb/main.go
```

## Kiroween Hackathon

This project is submitted to the Kiroween hackathon in the **Resurrection** category as a modern reimagining of classic network inspection tools like `netstat`, `ss`, `lsof`, and `iftop`.

## License

MIT OR Apache-2.0

## Credits

Built with:
- [tview](https://github.com/rivo/tview) - TUI framework with rich widgets
- [tcell](https://github.com/gdamore/tcell) - Low-level terminal control
- Kiro AI - Spec-driven development assistant

**Note:** Keyboard-only interface. No mouse support by design.
