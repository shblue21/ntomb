# ntomb - Requirements Specification

## Overview

ntomb is a Linux terminal application that visualizes network connections from a process-centric perspective. Unlike traditional tools like netstat or ss that show flat lists of connections, ntomb places a target process at the center and displays its network connections as a visual graph in the terminal. This makes it immediately clear which remote endpoints a process is communicating with, helping developers, SREs, and security engineers quickly identify suspicious connections, debug network issues, or understand service dependencies.

The tool combines practical network inspection with a subtle Halloween theme (coffins, tombstones, ghosts, skulls) that makes connection states visually distinct while maintaining readability and professional utility. It's designed for the Kiroween hackathon's Resurrection category as a modern reimagining of classic network inspection tools.

## User Personas & Use Cases

### Personas

**Linux SRE (Site Reliability Engineer)**
- Manages production servers and needs to quickly diagnose which services are talking to which endpoints
- Often works in SSH sessions with limited terminal sizes
- Values speed and clarity over fancy graphics

**Security Engineer**
- Investigates potential security incidents and suspicious network activity
- Needs to quickly spot unusual outbound connections or unexpected remote endpoints
- Requires accurate, real-time data about process network behavior

**Backend Developer**
- Debugging microservices and API integrations in development or staging environments
- Wants to understand service dependencies and connection patterns
- Needs a tool that works without complex setup or configuration

### Use Cases

**UC-1: Inspect all connections for a running service**
A developer notices their API service is slow and wants to see all active connections. They run `ntomb --process api-server` and immediately see a graph with the service in the center and all remote endpoints around it, grouped by IP or domain. They can quickly identify if the service is making unexpected calls or if connection counts are abnormal.

**UC-2: Spot suspicious outbound connections**
A security engineer receives an alert about potential data exfiltration from a web server. They run `ntomb --pid 1234` on the suspicious process and see connections to known endpoints plus one unusual connection to an unfamiliar IP in a foreign country. The visual layout makes the outlier immediately obvious compared to scanning through text output.

**UC-3: Debug connection state issues**
An SRE is investigating why a service has degraded performance. Using ntomb, they discover dozens of connections stuck in TIME_WAIT state (shown as "ghost" icons in Halloween mode), indicating the service isn't properly closing connections. This visual representation makes the problem obvious at a glance.

**UC-4: Understand service dependencies**
A developer joins a new team and needs to understand what external services their application depends on. They run ntomb on the main process and see connections to databases, cache servers, and external APIs, all visually organized. This gives them a quick mental model of the system architecture.

**UC-5: Demo and documentation**
For the hackathon presentation and README screenshots, the team needs to show ntomb's capabilities without requiring judges to have specific services running. They use `ntomb --demo` mode which displays a realistic but fake network graph, demonstrating all features including the Halloween theme.

**UC-6: Monitor connection lifecycle**
An engineer wants to watch how connections evolve over time. They run ntomb with auto-refresh and observe connections transitioning between states (ESTABLISHED → FIN_WAIT → TIME_WAIT), with visual indicators (colors, icons) making state changes immediately noticeable.

## Functional Requirements

**FR-1: Process Selection by PID**
WHEN the user invokes ntomb with `--pid <number>`, THEN the system SHALL identify the process with that PID and display its network connections.

**FR-2: Process Selection by Name**
WHEN the user invokes ntomb with `--process <name>`, THEN the system SHALL search for processes matching that name and either display connections for a single match or prompt the user to select from multiple matches.

**FR-3: Connection Scanning**
The system SHALL scan current TCP and UDP connections from the Linux system using /proc/net/tcp, /proc/net/udp, or the ss command, and SHALL associate connections with the target process.

**FR-4: Process-Centric Graph Model**
The system SHALL build a graph where the target process is the central node and remote endpoints (IP addresses or resolved domains) are surrounding nodes, with edges representing active connections.

**FR-5: Terminal UI Display**
The system SHALL render the network graph in a terminal user interface that works in standard terminal sizes (minimum 80x24) and adapts to larger terminals for better visualization.

**FR-6: Connection State Visualization**
The system SHALL display connection states (ESTABLISHED, LISTEN, TIME_WAIT, CLOSE_WAIT, FIN_WAIT, SYN_SENT, SYN_RECV) with distinct visual indicators (colors, icons, or labels) so users can quickly identify connection health.

**FR-7: Interactive Navigation**
The system SHALL support keyboard navigation including arrow keys to select nodes, Tab to switch between panels, 'r' to refresh data, and 'q' to quit the application.

**FR-8: Connection Details Panel**
WHEN the user selects a remote endpoint node, THEN the system SHALL display detailed information including IP address, port, protocol, connection state, and timestamps in a side panel.

**FR-9: Halloween Theme Mode**
The system SHALL support a `--theme=halloween` option that decorates the UI with Halloween-themed icons (coffin/tombstone for the main process, pumpkins for ESTABLISHED connections, ghosts for TIME_WAIT, skulls for closed/failed connections) while maintaining readability and professional utility.

**FR-10: Default Theme Mode**
The system SHALL provide a default theme without Halloween decorations for users who prefer a more traditional appearance, using clear text labels and standard terminal colors.

**FR-11: Demo Mode**
The system SHALL support a `--demo` flag that generates and displays a realistic fake network graph without requiring actual network connections or elevated privileges, suitable for demonstrations and screenshots.

**FR-12: Node Grouping and Limiting**
WHEN a process has many connections to the same remote IP or subnet, THEN the system SHALL group them intelligently to prevent UI overcrowding while still showing connection counts and allowing drill-down to individual connections.

## Non-Functional Requirements

**NFR-1: Performance**
The system SHALL scan connections and render the UI within 2 seconds on a typical Linux server, and SHALL support auto-refresh at configurable intervals (default 5 seconds) without noticeable lag or flicker.

**NFR-2: Security and Privileges**
The system SHALL operate in read-only mode without modifying any system state, SHALL request only the minimum privileges necessary to read /proc and network information, and SHALL clearly document any privilege requirements (e.g., running as the process owner or root for full connection visibility).

**NFR-3: Portability**
The system SHALL run on common Linux distributions (Ubuntu, Debian, RHEL, Fedora, Arch) without requiring distribution-specific modifications, and SHALL gracefully handle differences in /proc filesystem layouts or available system tools.

**NFR-4: Terminal Compatibility**
The system SHALL work correctly in standard terminal emulators (xterm, gnome-terminal, iTerm2, tmux, screen) with minimum size 80x24, SHALL detect terminal capabilities and adapt rendering accordingly, and SHALL provide enhanced visualization in larger terminals (120x40 or bigger).

**NFR-5: Error Handling and Messages**
The system SHALL provide clear, actionable error messages when processes are not found, permissions are insufficient, or system information is unavailable, and SHALL log errors to stderr without cluttering the main UI.

**NFR-6: Resource Usage**
The system SHALL use minimal CPU and memory resources (< 50MB RAM, < 5% CPU on idle), SHALL not leak memory during long-running sessions with auto-refresh, and SHALL clean up resources properly on exit.

**NFR-7: Observability**
The system SHALL support a `--verbose` or `--debug` flag that outputs detailed logging about connection scanning, graph building, and rendering decisions to help developers and users troubleshoot issues.

**NFR-8: User Experience**
The system SHALL provide intuitive keyboard shortcuts with an on-screen help panel (accessible via 'h' or '?'), SHALL use consistent visual language throughout the UI, and SHALL make the most important information (connection states, suspicious patterns) immediately visible without requiring navigation.
