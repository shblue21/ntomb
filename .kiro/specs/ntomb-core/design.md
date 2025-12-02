# ntomb - Design Specification

## Architecture Overview

ntomb is implemented as a single static binary written in **Rust**, targeting Linux systems. Rust provides memory safety, excellent performance, zero-cost abstractions, and a rich ecosystem for terminal UI development (Ratatui) and system interaction.

### Invocation
```bash
# By PID
ntomb --pid 1234

# By process name
ntomb --process nginx

# With Halloween theme
ntomb --pid 1234 --theme=halloween

# Demo mode for presentations
ntomb --demo

# With auto-refresh
ntomb --pid 1234 --refresh 5
```

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         ntomb CLI                            │
│                      (src/main.rs)                           │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
┌─────────────┐  ┌──────────────┐  ┌──────────┐
│  Process    │  │  NetScan     │  │  Theme   │
│  Module     │  │  Module      │  │  Module  │
└──────┬──────┘  └──────┬───────┘  └────┬─────┘
       │                │               │
       └────────┬───────┘               │
                ▼                       │
        ┌──────────────┐                │
        │    Graph     │                │
        │   Module     │                │
        └──────┬───────┘                │
               │                        │
               └────────┬───────────────┘
                        ▼
                ┌──────────────┐
                │     TUI      │
                │   Module     │
                │  (Ratatui)   │
                └──────────────┘
```

**Data Flow:**
1. CLI parses arguments and determines target process
2. Process Scanner resolves PID/name and gathers process metadata
3. Connection Scanner reads network connections from /proc or ss
4. Graph Builder creates a process-centric graph model
5. TUI Renderer displays the graph with the selected theme
6. User interactions trigger refresh or navigation updates

## Core Modules

### src/process/
- **Responsibility:** Identify and gather information about target processes
- **Key functions:**
  - `find_by_pid(pid: u32) -> Result<Process>` - Look up process by PID
  - `find_by_name(name: &str) -> Result<Vec<Process>>` - Search processes by name
  - `get_process_info(pid: u32) -> Result<ProcessInfo>` - Read /proc/{pid}/cmdline, /proc/{pid}/exe, etc.
- **Data sources:** /proc filesystem (/proc/{pid}/stat, /proc/{pid}/cmdline)
- **Error handling:** Handle missing processes, permission errors, zombie processes

### src/netscan/
- **Responsibility:** Gather current TCP/UDP connections for a specific process
- **Key functions:**
  - `scan_tcp_connections(pid: u32) -> Result<Vec<Connection>>` - Read TCP connections
  - `scan_udp_connections(pid: u32) -> Result<Vec<Connection>>` - Read UDP connections
  - `parse_proc_net(path: &Path, pid: u32) -> Result<Vec<Connection>>` - Parse /proc/net/tcp, /proc/net/udp
  - `resolve_hostname(ip: IpAddr) -> Option<String>` - Optional reverse DNS lookup
- **Data sources:** /proc/net/tcp, /proc/net/tcp6, /proc/net/udp, /proc/net/udp6, or ss command output
- **Performance:** Cache DNS lookups, limit resolution attempts to avoid blocking
- **Filtering:** Only return connections belonging to the target PID

### src/graph/
- **Responsibility:** Transform process and connection data into a graph model suitable for visualization
- **Key functions:**
  - `build_graph(process: &Process, connections: &[Connection]) -> Result<Graph>` - Create graph from raw data
  - `group_connections(connections: &[Connection]) -> Result<Vec<RemoteNode>>` - Group by IP/subnet/domain
  - `calculate_layout(graph: &Graph, terminal_size: Size) -> Result<Layout>` - Position nodes for rendering
  - `detect_suspicious(connection: &Connection) -> SuspicionLevel` - Flag unusual patterns
- **Grouping logic:**
  - Group multiple connections to same IP into single node with connection count
  - Optionally group by /24 subnet if too many unique IPs
  - Limit total displayed nodes to prevent overcrowding (e.g., top 20 by connection count)
- **Layout algorithm:** Simple radial layout with process at center, remote nodes in a circle or grid around it

### src/tui/
- **Responsibility:** Render the network graph in a terminal UI using Ratatui
- **Key functions:**
  - `App::new(graph: Graph, theme: Box<dyn Theme>) -> App` - Create TUI application
  - `App::update(&mut self, event: Event) -> Result<()>` - Handle updates and input
  - `App::render(&self, frame: &mut Frame)` - Render the UI
  - `draw_graph(frame: &mut Frame, graph: &Graph, theme: &dyn Theme)` - Draw the network map
  - `draw_details_panel(frame: &mut Frame, selected: &Node)` - Show connection details
- **Layout:**
  - Main area: Network graph (70-80% of screen width)
  - Right panel: Details for selected node (20-30% of screen width)
  - Bottom bar: Status line with refresh time, connection count, help hint
- **Rendering:**
  - Use Ratatui's widget system for state management
  - Use Ratatui's styling for colors and layout
  - Support keyboard input for navigation via crossterm

### src/theme/
- **Responsibility:** Define visual appearance (colors, icons, symbols) for different themes
- **Key structures:**
  - `Theme` trait with color palette and icon mappings
  - `ThemeType` enum: Default, Halloween
  - Icon/symbol definitions for each connection state and node type
- **Default theme:**
  - Process node: `[PROC]` or `●` with bright color
  - Remote nodes: `○` with standard colors
  - ESTABLISHED: Green
  - LISTEN: Blue
  - TIME_WAIT: Yellow
  - CLOSE_WAIT: Orange
  - Failed/Closed: Red
- **Halloween theme:**
  - Process node: `⚰` (coffin) or `🪦` (tombstone) with purple/orange
  - ESTABLISHED: `🎃` (pumpkin) with orange
  - LISTEN: `👻` (ghost) with white/cyan
  - TIME_WAIT: `👻` (ghost, faded) with gray
  - CLOSE_WAIT: `💀` (skull) with yellow
  - Failed/Closed: `☠` (skull and crossbones) with red
  - Background: Subtle dark purple/black tones
- **Readability:** Ensure sufficient contrast, provide fallback ASCII symbols for terminals without Unicode support

## Data Model

### Process
```rust
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub exe_path: PathBuf,
    pub user: String,
}
```

### Connection
```rust
pub struct Connection {
    pub protocol: Protocol,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub state: ConnectionState,
    pub pid: u32,
    pub inode: u64,
    pub timestamp: Option<Instant>,
    pub suspicious: SuspicionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Established,
    Listen,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    CloseWait,
    LastAck,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspicionLevel {
    Normal,
    Suspicious,
    HighRisk,
}
```

### GraphNode
```rust
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub position: Position,
    pub connections: Vec<String>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Process,
    RemoteEndpoint,
    RemoteGroup,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub struct NodeMetadata {
    pub ip: Option<IpAddr>,
    pub hostname: Option<String>,
    pub port: u16,
    pub connection_count: usize,
    pub suspicious: bool,
}
```

### GraphEdge
```rust
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub state: ConnectionState,
    pub weight: usize,
}
```

### Graph
```rust
pub struct Graph {
    pub process_node: GraphNode,
    pub remote_nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

pub struct GraphMetadata {
    pub total_connections: usize,
    pub by_state: HashMap<ConnectionState, usize>,
    pub suspicious_count: usize,
    pub last_updated: Instant,
}
```

## TUI Layout & Interaction

### Screen Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ ntomb - Process: nginx (PID: 1234)                    [? for help] [q] │
├──────────────────────────────────────────┬──────────────────────────────┤
│                                          │  Selected Node               │
│              192.168.1.100:443           │  ─────────────               │
│                    ○                     │  IP: 192.168.1.100           │
│                     \                    │  Port: 443                   │
│                      \                   │  Hostname: api.example.com   │
│         10.0.0.5:80   ⚰ nginx            │  Protocol: TCP               │
│              ○────────●────────○         │  State: ESTABLISHED          │
│                      /          \        │  Connections: 3              │
│                     /            \       │                              │
│                    ○              ○      │  Details:                    │
│            8.8.8.8:53      10.0.1.20:3306│  - 192.168.1.100:443 → EST  │
│                                          │  - 192.168.1.100:8080 → EST │
│                                          │  - 192.168.1.100:8443 → EST │
│                                          │                              │
│  Connections: 12 (8 EST, 2 WAIT, 2 LST) │                              │
│  Suspicious: 0                           │                              │
├──────────────────────────────────────────┴──────────────────────────────┤
│ ↑↓←→: Navigate | Tab: Switch panel | r: Refresh | h: Help | q: Quit    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Bindings

- **Arrow keys (↑↓←→):** Navigate between nodes in the graph
- **Tab:** Switch focus between graph panel and details panel
- **Enter:** Expand grouped node to show individual connections
- **r:** Refresh connection data
- **h or ?:** Toggle help panel
- **q or Esc:** Quit application
- **t:** Toggle theme (default ↔ halloween)
- **f:** Toggle filter (show all / hide TIME_WAIT / show only suspicious)
- **+/-:** Zoom in/out (adjust node spacing if terminal is large)

### Visual Application of Halloween Theme

**Without breaking readability:**
- Icons are used alongside or instead of text labels, not as replacements for critical information
- Colors maintain sufficient contrast (WCAG AA minimum)
- State information is redundant: shown via icon, color, AND text label
- ASCII fallback mode for terminals without Unicode support
- Theme can be toggled at runtime with 't' key

**Example Halloween rendering:**
```
        🎃 api.example.com:443
              (ESTABLISHED)
                  │
                  │
    👻 cache:6379 ⚰ nginx 🎃 db:3306
      (LISTEN)     (PID)    (ESTABLISHED)
                  │
                  │
              💀 old-api:80
              (CLOSE_WAIT)
```

**Same in default theme:**
```
        ● api.example.com:443
              (ESTABLISHED)
                  │
                  │
    ○ cache:6379  ● nginx  ● db:3306
      (LISTEN)    (PID)    (ESTABLISHED)
                  │
                  │
              ○ old-api:80
              (CLOSE_WAIT)
```

## Integration Points for Kiro & MCP

### Spec-Driven Development with Kiro

This design document, along with requirements.md and tasks.md, forms the specification for Kiro-assisted development:

1. **Initial scaffolding:** Kiro can generate the Rust project structure, Cargo.toml, and module stubs based on the architecture
2. **Module implementation:** Each core module (process, netscan, graph, tui, theme) can be implemented iteratively with Kiro's assistance
3. **Test generation:** Kiro can create unit tests for each module based on the functional requirements
4. **Refactoring:** As the design evolves, Kiro can help refactor code to maintain consistency with the spec

### MCP Server for ntomb (Future Enhancement)

An MCP server could expose ntomb's functionality to other tools and AI assistants:

**Tools:**
- `ntomb_list_processes()` - List all processes with network connections
- `ntomb_list_connections(pid: u32)` - Get connections for a specific process
- `ntomb_get_graph(pid: u32, format: &str)` - Get the network graph (format: "json" or "text")
- `ntomb_summarize_suspicious(pid: u32)` - Analyze and report suspicious connections
- `ntomb_compare_snapshots(pid: u32, snapshot1: Instant, snapshot2: Instant)` - Compare connection patterns over time

**Resources:**
- `ntomb://process/{pid}` - Process information and current connections
- `ntomb://connection/{pid}/{remote_addr}` - Specific connection details
- `ntomb://graph/{pid}` - Full graph representation

**Use cases:**
- AI assistants can query ntomb to help debug network issues
- Security tools can integrate ntomb for automated connection analysis
- Monitoring systems can use ntomb's suspicious connection detection

**Implementation notes:**
- MCP server would be a separate binary or feature flag in ntomb
- Uses the same core modules (process, netscan, graph)
- Returns structured JSON instead of rendering TUI
- Can run in daemon mode for continuous monitoring

This MCP integration is optional for the initial hackathon submission but demonstrates how ntomb fits into the broader Kiro ecosystem.
