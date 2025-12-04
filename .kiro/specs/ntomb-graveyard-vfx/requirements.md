# Requirements Document: Graveyard VFX & Kiroween Enhancement

## Introduction

This feature upgrades **The Graveyard (Network Topology)** panel to be more visually striking, data-meaningful, and strongly aligned with the Kiroween (Halloween/Undead) concept. The goal is to transform the network map from a functional display into an immersive visualization that helps SREs and security engineers instantly grasp network health while enjoying the necromancer theme.

All enhancements respect the existing visual-design and security-domain steering documents:
- Color palette: Neon Purple, Pumpkin Orange, Blood Red, Toxic Green, Bone White
- Tone: Calm and informative, never fear-mongering
- Principle: Read-only observation, accessibility-first, all effects toggleable

## Glossary

- **Graveyard**: The network topology canvas showing HOST and connected endpoints
- **Latency Ring**: Concentric circle around HOST representing latency buckets
- **Spirit Flow / Edge Particle**: Animated dots moving along connection lines
- **Endpoint**: A remote IP:port that HOST connects to or receives connections from
- **Heavy Talker**: An endpoint with connection count above a defined threshold
- **Kiroween Overdrive**: Enhanced Halloween-themed visual mode (optional)
- **RFC1918**: Private IP address ranges (10.x, 172.16-31.x, 192.168.x)

## Requirements

### Requirement 1: Latency Rings

**User Story:** As an SRE, I want to see endpoints positioned by latency distance from HOST, so that I can instantly identify slow connections without reading numbers.

#### Acceptance Criteria

1. WHEN the Graveyard renders THEN the ntomb system SHALL display 2-3 concentric rings around the central HOST node
2. WHEN an endpoint has low latency (< 50ms) THEN the ntomb system SHALL position it on or near the innermost ring
3. WHEN an endpoint has medium latency (50-200ms) THEN the ntomb system SHALL position it on or near the middle ring
4. WHEN an endpoint has high latency (> 200ms) THEN the ntomb system SHALL position it on or near the outermost ring
5. WHEN latency data is unavailable for an endpoint THEN the ntomb system SHALL fall back to the existing radial layout without rings
6. WHEN displaying rings THEN the ntomb system SHALL use subtle Bone White color to avoid visual clutter

### Requirement 2: Edge Particle / Spirit Flow Animation

**User Story:** As a user, I want to see animated particles flowing along connection lines, so that I can perceive the network as "alive" and understand traffic direction.

#### Acceptance Criteria

1. WHEN animations are enabled THEN the ntomb system SHALL render moving dots along each connection edge
2. WHEN rendering particles THEN the ntomb system SHALL use pulse_phase to calculate particle position (0.0 to 1.0 along the edge)
3. WHEN displaying particles THEN the ntomb system SHALL use colors from the approved palette (Neon Purple for normal, Toxic Green for healthy, Pumpkin Orange for warning)
4. WHEN the user presses 'A' key THEN the ntomb system SHALL toggle animations on/off
5. WHEN animations are disabled THEN the ntomb system SHALL display static connection lines without particles
6. WHEN animations are disabled THEN the ntomb system SHALL maintain full readability of the network map

### Requirement 3: Endpoint Type Icons and Colors

**User Story:** As a security engineer, I want endpoints visually categorized by type (internal/external/listener/heavy-talker), so that I can quickly spot unusual patterns.

#### Acceptance Criteria

1. WHEN an endpoint IP is RFC1918 private THEN the ntomb system SHALL display 🪦 icon with Bone White color
2. WHEN an endpoint IP is public (non-RFC1918) THEN the ntomb system SHALL display 🎃 icon with Pumpkin Orange color
3. WHEN an endpoint is LISTEN-only (local server socket) THEN the ntomb system SHALL display 🕯 icon with Neon Purple color
4. WHEN an endpoint has connection count in top 5 THEN the ntomb system SHALL display 👑 badge indicating "heavy talker"
5. WHEN an endpoint is localhost (127.0.0.1 or ::1) THEN the ntomb system SHALL display ⚰️ icon with Toxic Green color
6. WHEN the user presses 't' key THEN the ntomb system SHALL toggle endpoint text labels on/off

#### Icon/Color Mapping Table

| Endpoint Type | Icon | Primary Color | Notes |
|---------------|------|---------------|-------|
| Localhost | ⚰️ | Toxic Green | Local loopback connections |
| RFC1918 Private | 🪦 | Bone White | Internal network endpoints |
| Public IP | 🎃 | Pumpkin Orange | External/internet endpoints |
| LISTEN-only | 🕯 | Neon Purple | Local server sockets |
| Heavy Talker (top 5) | +👑 | (base color) | Badge added to base icon |
| Zombie/Error state | 💀 | Blood Red | Connection issues |

### Requirement 4: Kiroween Overdrive Mode

**User Story:** As a user who enjoys the Halloween theme, I want an optional "overdrive" mode that enhances the spooky aesthetics without compromising usability.

#### Acceptance Criteria

1. WHEN the user presses 'H' key THEN the ntomb system SHALL toggle Kiroween Overdrive mode on/off
2. WHEN Overdrive is enabled AND connection is ESTABLISHED THEN the ntomb system SHALL display 👻 ghost icon alongside status
3. WHEN Overdrive is enabled AND endpoint has high latency THEN the ntomb system SHALL display 🔥🎃 fire-pumpkin combination
4. WHEN Overdrive is enabled AND endpoint has many CLOSE_WAIT/TIME_WAIT THEN the ntomb system SHALL display 💀 skull tag
5. WHEN Overdrive is enabled THEN the ntomb system SHALL update status bar with themed text (e.g., "Spirits: 128" instead of "Connections: 128")
6. WHEN Overdrive mode changes any text THEN the ntomb system SHALL maintain calm, informative tone per security-domain guidelines
7. WHEN the application starts THEN the ntomb system SHALL have Overdrive mode disabled by default

#### Overdrive Visual Mapping

| State | Normal Mode | Overdrive Mode |
|-------|-------------|----------------|
| ESTABLISHED (healthy) | 🟢 | 🟢👻 |
| High Latency | 🟠 | 🔥🎃 |
| CLOSE_WAIT storm | ⚠️ | 💀 |
| Connection count | "Connections: N" | "Spirits: N" |
| Healthy endpoint | "Alive" | "Haunting" |

### Requirement 5: Toggle Controls and Accessibility

**User Story:** As a user on a low-spec machine or SSH session, I want to disable visual effects while keeping full functionality, so that ntomb remains usable in any environment.

#### Acceptance Criteria

1. WHEN the user presses 'A' key THEN the ntomb system SHALL toggle all animations (particles, pulse effects) on/off
2. WHEN the user presses 'H' key THEN the ntomb system SHALL toggle Kiroween Overdrive mode on/off
3. WHEN the user presses 't' key THEN the ntomb system SHALL toggle endpoint text labels on/off
4. WHEN animations are disabled THEN the ntomb system SHALL render static graphics that convey the same information
5. WHEN running in a low-bandwidth SSH session THEN the ntomb system SHALL not cause noticeable lag with effects disabled
6. WHEN the status bar renders THEN the ntomb system SHALL display current toggle states (e.g., "[A:ON] [H:OFF] [t:ON]")
7. WHEN any toggle changes THEN the ntomb system SHALL apply the change immediately without restart

#### Keyboard Shortcuts Summary

| Key | Action | Default State |
|-----|--------|---------------|
| A | Toggle animations | ON |
| H | Toggle Kiroween Overdrive | OFF |
| t | Toggle endpoint labels | ON |

### Requirement 6: Performance Requirements

**User Story:** As a user monitoring a busy server, I want ntomb to remain responsive even with hundreds of connections.

#### Acceptance Criteria

1. WHEN displaying 200+ connections THEN the ntomb system SHALL maintain at least 10 FPS with animations enabled
2. WHEN displaying 200+ connections with animations disabled THEN the ntomb system SHALL maintain at least 30 FPS
3. WHEN rendering the Graveyard THEN the ntomb system SHALL limit visible endpoints to prevent canvas overflow
4. WHEN too many endpoints exist THEN the ntomb system SHALL aggregate or show "... and N more" indicator
5. WHEN CPU usage exceeds acceptable threshold THEN the ntomb system SHALL automatically reduce animation complexity
