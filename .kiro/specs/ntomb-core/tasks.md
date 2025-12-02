# ntomb - Implementation Tasks

## Phase 1: Bootstrapping

- [ ] Initialize Rust project with `cargo init`
- [ ] Set up project structure with modules (src/main.rs, src/process/, src/netscan/, src/graph/, src/tui/, src/theme/, src/demo/)
- [ ] Add dependencies to Cargo.toml: ratatui for TUI, crossterm for terminal handling, clap for CLI
- [ ] Implement CLI argument parsing with clap (--pid, --process, --theme, --demo, --refresh, --verbose)
- [ ] Add basic error handling patterns with thiserror and anyhow
- [ ] Create --demo flag that loads a hardcoded fake dataset for testing
- [ ] Set up logging with tracing or env_logger for --verbose mode
- [ ] Add README.md with basic usage examples and build instructions

## Phase 2: Process & Connection Collection

- [ ] Implement process::find_by_pid() to read /proc/{pid}/stat and /proc/{pid}/cmdline
- [ ] Implement process::find_by_name() to search /proc for matching process names
- [ ] Add process selection logic: if multiple matches, prompt user to choose
- [ ] Implement netscan::parse_proc_net() to read /proc/net/tcp and /proc/net/tcp6
- [ ] Add UDP support: parse /proc/net/udp and /proc/net/udp6
- [ ] Map inode numbers from /proc/net/* to PIDs via /proc/{pid}/fd/*
- [ ] Filter connections to only include those belonging to target PID
- [ ] Add connection state parsing (hex values to ConnectionState enum variants)
- [ ] Implement optional reverse DNS lookup with timeout (for hostname resolution)
- [ ] Create a small test script that validates connection scanning against known processes (e.g., sshd, systemd)
- [ ] Add error handling for permission issues and missing /proc entries

## Phase 3: Graph Building

- [ ] Define data structures: Process, Connection, GraphNode, GraphEdge, Graph
- [ ] Implement graph::build_graph() to create process node and remote nodes from connections
- [ ] Add connection grouping logic: group by remote IP, count connections per IP
- [ ] Implement node limiting: if > 20 unique IPs, show top 20 by connection count
- [ ] Add subnet grouping option: group by /24 if too many unique IPs
- [ ] Implement simple radial layout algorithm: position remote nodes in circle around process
- [ ] Add suspicious connection detection: flag unusual ports, private IP ranges, high connection counts
- [ ] Create GraphMetadata with connection statistics (total, by state, suspicious count)
- [ ] Write unit tests for graph building with various connection patterns
- [ ] Add layout calculation that adapts to terminal size

## Phase 4: TUI Rendering

- [ ] Set up Ratatui with crossterm backend initialization
- [ ] Create App struct to hold graph, selected node, current panel, theme
- [ ] Implement App::new() to initialize the TUI application
- [ ] Implement App::update() to handle events and input
- [ ] Implement App::render() to render the UI using Ratatui widgets
- [ ] Add process node rendering at center with label and icon
- [ ] Add remote node rendering in circle/grid around process
- [ ] Draw edges (lines) connecting process to remote nodes using box-drawing characters
- [ ] Implement details panel showing selected node information
- [ ] Add connection list in details panel with state, port, protocol
- [ ] Implement status bar at bottom with connection counts and help hint
- [ ] Add keyboard input handling: arrow keys for navigation, Tab for panel switching
- [ ] Implement node selection: highlight selected node, update details panel
- [ ] Add 'r' key for manual refresh of connection data
- [ ] Add 'q' and Esc keys for quitting application
- [ ] Implement 'h' or '?' key to toggle help panel overlay
- [ ] Add help panel with keyboard shortcuts and usage tips
- [ ] Test rendering in 80x24 terminal and verify readability
- [ ] Test rendering in larger terminals (120x40+) and verify layout scales

## Phase 5: Halloween Theme

- [ ] Create theme module with Theme trait and ThemeType enum
- [ ] Define default theme: colors, symbols, icons for each connection state
- [ ] Define Halloween theme: coffin/tombstone for process, pumpkins, ghosts, skulls
- [ ] Map ConnectionState to theme-specific icons and colors
- [ ] Implement Theme::node_icon() and Theme::state_color() methods
- [ ] Add Unicode support detection for terminal
- [ ] Implement ASCII fallback symbols for terminals without Unicode
- [ ] Integrate theme into App for node and edge styling using Ratatui styles
- [ ] Add 't' key to toggle between default and Halloween themes at runtime
- [ ] Test both themes for readability and contrast
- [ ] Ensure Halloween theme maintains professional utility (not just decorative)
- [ ] Capture screenshots of both themes for README and hackathon submission
- [ ] Create demo mode scenarios that showcase different connection states and themes

## Phase 6: Polish & Testing

- [ ] Add auto-refresh functionality with configurable interval (--refresh flag) using tokio or std::thread
- [ ] Implement connection state change detection (highlight new/changed connections)
- [ ] Add filter functionality: 'f' key to toggle showing/hiding TIME_WAIT connections
- [ ] Implement suspicious connection highlighting in graph view
- [ ] Add connection count badges on grouped nodes
- [ ] Optimize rendering performance: only redraw on changes
- [ ] Add memory profiling: run with auto-refresh for extended period
- [ ] Test on multiple Linux distributions (Ubuntu, Fedora, Arch)
- [ ] Test with various processes: web servers, databases, SSH, system services
- [ ] Test edge cases: process with no connections, process with 100+ connections
- [ ] Add error messages for common issues: process not found, insufficient permissions
- [ ] Write integration tests using demo mode
- [ ] Add CI/CD pipeline for automated testing and building (GitHub Actions)
- [ ] Create release build with optimizations: `cargo build --release`

## Phase 7: Documentation & Hackathon Prep

- [ ] Write comprehensive README.md with features, installation, usage, screenshots
- [ ] Add animated GIF or video demo showing ntomb in action
- [ ] Document Halloween theme and Kiroween hackathon connection
- [ ] Create ARCHITECTURE.md explaining design decisions and code structure
- [ ] Add inline code documentation and rustdoc comments
- [ ] Write CONTRIBUTING.md for potential future contributors
- [ ] Prepare hackathon submission: video demo, slides, talking points
- [ ] Capture screenshots for Resurrection category: comparison with netstat/ss/lsof
- [ ] Test demo mode thoroughly for hackathon judging (no network access required)
- [ ] Create example use cases and scenarios for presentation
- [ ] Prepare answers for common questions: Why Rust? Why process-centric? Why Halloween theme?

## Phase 8: Kiro Integration

- [ ] Review requirements.md, design.md, and tasks.md with Kiro for consistency
- [ ] Use Kiro to generate initial module implementations from specs
- [ ] Use Kiro to refactor code based on design patterns in design.md
- [ ] Use Kiro to generate unit tests based on functional requirements
- [ ] Use Kiro to optimize performance bottlenecks identified during testing
- [ ] Use Kiro to improve error messages and user experience
- [ ] Document how Kiro was used in development process for hackathon narrative

## Phase 9: MCP Integration (Optional/Future)

- [ ] Design MCP server API: tools and resources for ntomb
- [ ] Implement MCP server binary or feature flag in ntomb
- [ ] Add JSON output mode for graph and connection data
- [ ] Implement ntomb_list_processes() MCP tool
- [ ] Implement ntomb_list_connections(pid) MCP tool
- [ ] Implement ntomb_get_graph(pid) MCP tool
- [ ] Implement ntomb_summarize_suspicious(pid) MCP tool
- [ ] Add MCP resource handlers for process and connection URIs
- [ ] Test MCP integration with Kiro or other MCP clients
- [ ] Document MCP server usage and integration examples
- [ ] Consider daemon mode for continuous monitoring via MCP

## Phase 10: Post-Hackathon Enhancements (Backlog)

- [ ] Add support for filtering by protocol (TCP only, UDP only)
- [ ] Implement connection history tracking and timeline view
- [ ] Add export functionality: save graph as JSON, CSV, or image
- [ ] Implement comparison mode: compare two snapshots to see changes
- [ ] Add GeoIP lookup to show country/region for remote IPs
- [ ] Implement ASN lookup to identify cloud providers and networks
- [ ] Add packet capture integration (pcap) for deeper analysis
- [ ] Create web UI version using same core modules
- [ ] Add alerting: notify when suspicious connections appear
- [ ] Implement plugin system for custom connection analysis rules
- [ ] Add support for container networking (Docker, Kubernetes)
- [ ] Create ntomb-server for centralized monitoring of multiple hosts
