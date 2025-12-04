# Design Document: Graveyard VFX & Kiroween Enhancement

## Overview

This design transforms The Graveyard network topology panel into an immersive, data-rich visualization. The enhancements add:
1. **Latency Rings** - Spatial encoding of network delay
2. **Spirit Flow Particles** - Animated traffic direction indicators
3. **Endpoint Type Icons** - Visual categorization of connection types
4. **Kiroween Overdrive** - Optional Halloween theme enhancement
5. **Toggle System** - Accessibility controls for all effects

All designs align with `visual-design.md` (color palette, clarity-first) and `security-domain.md` (calm tone, read-only).

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        AppState                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ graveyard_settings: GraveyardSettings                    │   │
│  │   - animations_enabled: bool                             │   │
│  │   - labels_enabled: bool                                 │   │
│  │   - overdrive_enabled: bool                              │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ latency_buckets: LatencyConfig                           │   │
│  │   - low_threshold_ms: u64 (default: 50)                  │   │
│  │   - high_threshold_ms: u64 (default: 200)                │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Graveyard Rendering Pipeline                   │
│                                                                  │
│  1. Collect Endpoints ──► 2. Classify Types ──► 3. Assign Rings │
│         │                        │                     │         │
│         ▼                        ▼                     ▼         │
│  4. Calculate Coords ──► 5. Draw Rings ──► 6. Draw Edges        │
│         │                        │                     │         │
│         ▼                        ▼                     ▼         │
│  7. Draw Nodes ──────► 8. Draw Particles ──► 9. Draw Labels     │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. GraveyardSettings Struct (src/app.rs)

```rust
/// Visual settings for the Graveyard panel
#[derive(Debug, Clone)]
pub struct GraveyardSettings {
    /// Enable particle animations on edges
    pub animations_enabled: bool,
    
    /// Show text labels on endpoints
    pub labels_enabled: bool,
    
    /// Enable Kiroween Overdrive theme
    pub overdrive_enabled: bool,
}

impl Default for GraveyardSettings {
    fn default() -> Self {
        Self {
            animations_enabled: true,
            labels_enabled: true,
            overdrive_enabled: false,  // Off by default
        }
    }
}
```

### 2. LatencyConfig Struct (src/app.rs)

```rust
/// Configuration for latency ring thresholds
#[derive(Debug, Clone)]
pub struct LatencyConfig {
    /// Threshold for "low latency" bucket (ms)
    pub low_threshold_ms: u64,
    
    /// Threshold for "high latency" bucket (ms)  
    pub high_threshold_ms: u64,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            low_threshold_ms: 50,
            high_threshold_ms: 200,
        }
    }
}

/// Latency bucket classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LatencyBucket {
    Low,      // < 50ms - innermost ring
    Medium,   // 50-200ms - middle ring
    High,     // > 200ms - outermost ring
    Unknown,  // No latency data - use default position
}
```

### 3. EndpointType Enum (src/ui.rs)

```rust
/// Classification of endpoint types for visual rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndpointType {
    Localhost,      // 127.0.0.1, ::1
    Private,        // RFC1918: 10.x, 172.16-31.x, 192.168.x
    Public,         // All other IPs
    ListenOnly,     // Local server sockets (no remote)
}

impl EndpointType {
    /// Get icon for this endpoint type
    pub fn icon(&self, overdrive: bool) -> &'static str {
        match self {
            Self::Localhost => "⚰️",
            Self::Private => "🪦",
            Self::Public => if overdrive { "🎃👻" } else { "🎃" },
            Self::ListenOnly => "🕯",
        }
    }
    
    /// Get primary color for this endpoint type
    pub fn color(&self) -> Color {
        match self {
            Self::Localhost => TOXIC_GREEN,
            Self::Private => BONE_WHITE,
            Self::Public => PUMPKIN_ORANGE,
            Self::ListenOnly => NEON_PURPLE,
        }
    }
}
```

### 4. AppState Extension

```rust
pub struct AppState {
    // ... existing fields ...
    
    /// Graveyard visual settings
    pub graveyard_settings: GraveyardSettings,
    
    /// Latency bucket configuration
    pub latency_config: LatencyConfig,
}
```

## Data Models

### Endpoint Classification Logic

```rust
fn classify_endpoint(ip: &str) -> EndpointType {
    if ip == "127.0.0.1" || ip == "::1" || ip == "0.0.0.0" {
        return EndpointType::Localhost;
    }
    
    // Parse IPv4 for RFC1918 check
    if let Ok(addr) = ip.parse::<Ipv4Addr>() {
        let octets = addr.octets();
        
        // 10.0.0.0/8
        if octets[0] == 10 {
            return EndpointType::Private;
        }
        
        // 172.16.0.0/12
        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
            return EndpointType::Private;
        }
        
        // 192.168.0.0/16
        if octets[0] == 192 && octets[1] == 168 {
            return EndpointType::Private;
        }
    }
    
    EndpointType::Public
}

fn is_heavy_talker(conn_count: usize, all_counts: &[usize]) -> bool {
    // Top 5 by connection count
    let mut sorted = all_counts.to_vec();
    sorted.sort_by(|a, b| b.cmp(a));
    
    if sorted.len() >= 5 {
        conn_count >= sorted[4]
    } else {
        conn_count >= sorted.last().copied().unwrap_or(0)
    }
}
```

## Latency Rings Implementation

### Canvas Coordinate System

```
        Canvas: 0-100 virtual space
        
              (50, 90)
                 │
    ┌────────────┼────────────┐
    │            │            │
    │   ┌────────┼────────┐   │  Outer Ring (High Latency)
    │   │  ┌─────┼─────┐  │   │  r = 35
    │   │  │     │     │  │   │
(10,50)─┼──┼─────●─────┼──┼───(90,50)
    │   │  │   HOST    │  │   │  Middle Ring (Medium)
    │   │  │  (50,50)  │  │   │  r = 25
    │   │  └───────────┘  │   │
    │   └─────────────────┘   │  Inner Ring (Low Latency)
    │                         │  r = 15
    └─────────────────────────┘
              (50, 10)
```

### Ring Drawing

```rust
const RING_RADII: [f64; 3] = [15.0, 25.0, 35.0];  // Inner, Middle, Outer
const HOST_CENTER: (f64, f64) = (50.0, 50.0);

fn draw_latency_rings(ctx: &mut Context) {
    for (i, radius) in RING_RADII.iter().enumerate() {
        let opacity = 0.3 - (i as f32 * 0.08);  // Fade outer rings
        
        // Draw ring as series of points (Braille approximation)
        for angle in (0..360).step_by(5) {
            let rad = (angle as f64).to_radians();
            let x = HOST_CENTER.0 + radius * rad.cos();
            let y = HOST_CENTER.1 + radius * rad.sin();
            
            ctx.print(x, y, Span::styled("·", 
                Style::default().fg(BONE_WHITE)));
        }
    }
}
```

### Endpoint Position Calculation

```rust
fn calculate_endpoint_position(
    endpoint_idx: usize,
    total_endpoints: usize,
    latency_bucket: LatencyBucket,
) -> (f64, f64) {
    // Distribute endpoints evenly around the ring
    let angle = (endpoint_idx as f64 / total_endpoints as f64) * 2.0 * PI;
    
    // Select ring radius based on latency
    let radius = match latency_bucket {
        LatencyBucket::Low => RING_RADII[0],      // 15
        LatencyBucket::Medium => RING_RADII[1],   // 25
        LatencyBucket::High => RING_RADII[2],     // 35
        LatencyBucket::Unknown => RING_RADII[1],  // Default to middle
    };
    
    // Add small random offset to prevent overlap
    let jitter = (endpoint_idx % 3) as f64 * 2.0 - 2.0;
    
    let x = HOST_CENTER.0 + (radius + jitter) * angle.cos();
    let y = HOST_CENTER.1 + (radius + jitter) * angle.sin();
    
    (x.clamp(5.0, 95.0), y.clamp(5.0, 95.0))
}
```

## Edge Particle Animation

### Particle Position Calculation

```rust
/// Calculate particle position along edge using pulse_phase
fn particle_position(
    start: (f64, f64),
    end: (f64, f64),
    pulse_phase: f32,
    particle_offset: f32,  // 0.0, 0.33, 0.66 for multiple particles
) -> (f64, f64) {
    // t ranges from 0.0 to 1.0 along the edge
    let t = (pulse_phase + particle_offset) % 1.0;
    
    let x = start.0 + (end.0 - start.0) * t as f64;
    let y = start.1 + (end.1 - start.1) * t as f64;
    
    (x, y)
}

fn draw_edge_with_particles(
    ctx: &mut Context,
    start: (f64, f64),
    end: (f64, f64),
    pulse_phase: f32,
    animations_enabled: bool,
    edge_color: Color,
) {
    // Draw base edge line
    ctx.draw(&CanvasLine {
        x1: start.0, y1: start.1,
        x2: end.0, y2: end.1,
        color: edge_color,
    });
    
    // Draw particles if animations enabled
    if animations_enabled {
        for offset in [0.0, 0.33, 0.66] {
            let (px, py) = particle_position(start, end, pulse_phase, offset);
            ctx.print(px, py, Span::styled("●", 
                Style::default().fg(TOXIC_GREEN)));
        }
    }
}
```

### Animation Timing

- Uses existing `pulse_phase` from AppState (0.0 to 1.0, cycles every ~2 seconds)
- 3 particles per edge, evenly spaced (0%, 33%, 66% along edge)
- Particles move from HOST outward (or inward for incoming connections)

## Kiroween Overdrive Mode

### Visual Transformations

```rust
fn get_status_text(state: ConnectionState, overdrive: bool) -> &'static str {
    match (state, overdrive) {
        (ConnectionState::Established, false) => "🟢 Alive",
        (ConnectionState::Established, true) => "🟢👻 Haunting",
        
        (ConnectionState::Listen, false) => "🟣 Listening",
        (ConnectionState::Listen, true) => "🕯 Summoning",
        
        (ConnectionState::TimeWait, false) => "⚠️ Closing",
        (ConnectionState::TimeWait, true) => "💀 Fading",
        
        (ConnectionState::CloseWait, false) => "⚠️ Stuck",
        (ConnectionState::CloseWait, true) => "💀 Trapped",
        
        _ => "❓ Unknown",
    }
}

fn get_stats_label(overdrive: bool) -> &'static str {
    if overdrive {
        "Spirits"
    } else {
        "Connections"
    }
}
```

### Tone Guidelines (per security-domain.md)

Even in Overdrive mode:
- ✅ "Spirits: 128" (playful)
- ✅ "Haunting" (thematic but calm)
- ❌ "DANGER! ZOMBIE ATTACK!" (fear-mongering)
- ❌ "System compromised!" (absolute claims)

## Key Bindings and Status Bar

### Keyboard Handler Updates (src/main.rs)

```rust
KeyCode::Char('a') | KeyCode::Char('A') => {
    app.graveyard_settings.animations_enabled = 
        !app.graveyard_settings.animations_enabled;
}
KeyCode::Char('h') | KeyCode::Char('H') => {
    app.graveyard_settings.overdrive_enabled = 
        !app.graveyard_settings.overdrive_enabled;
}
KeyCode::Char('t') | KeyCode::Char('T') => {
    app.graveyard_settings.labels_enabled = 
        !app.graveyard_settings.labels_enabled;
}
```

### Status Bar Display

```
💀 Q:R.I.P | ↑↓:Navigate | P:Focus | +/-:Speed | [A:ON] [H:OFF] [t:ON]
```

Toggle indicators:
- `[A:ON]` - Animations enabled (Toxic Green)
- `[A:OFF]` - Animations disabled (Bone White)
- `[H:ON]` - Overdrive enabled (Pumpkin Orange)
- `[H:OFF]` - Overdrive disabled (Bone White)
- `[t:ON]` - Labels enabled (Neon Purple)
- `[t:OFF]` - Labels disabled (Bone White)

## Rendering Pipeline

### Complete Flow

```
render_network_map(f, area, app)
│
├─► 1. Filter connections (Host/Process mode)
│
├─► 2. Aggregate endpoints
│   └─► Group by remote IP, count connections
│
├─► 3. Classify each endpoint
│   ├─► EndpointType (Localhost/Private/Public/Listen)
│   ├─► LatencyBucket (Low/Medium/High/Unknown)
│   └─► HeavyTalker check (top 5)
│
├─► 4. Draw latency rings (if any endpoint has latency data)
│
├─► 5. Calculate endpoint positions
│   └─► Ring-based or radial fallback
│
├─► 6. Draw edges (HOST ↔ endpoints)
│   └─► With particles if animations_enabled
│
├─► 7. Draw endpoint nodes
│   ├─► Icon based on EndpointType
│   ├─► Color based on state
│   └─► 👑 badge if heavy talker
│
└─► 8. Draw labels (if labels_enabled)
    └─► IP:port text near each node
```

## Error Handling

| Situation | Handling |
|-----------|----------|
| No latency data available | Fall back to radial layout, hide rings |
| Too many endpoints (> 50) | Show top 30 + "... and N more" |
| Animation causes lag | Auto-reduce particle count |
| Invalid IP format | Classify as Public (safe default) |

## Testing Strategy

### Unit Tests

1. `test_classify_endpoint` - Verify RFC1918 detection
2. `test_latency_bucket` - Verify threshold logic
3. `test_heavy_talker` - Verify top-N calculation
4. `test_particle_position` - Verify animation math

### Integration Tests

1. Toggle state persistence across frames
2. Mode compatibility (Host + Overdrive, Process + Animations)
3. Performance with 100+ endpoints

### Manual Testing

1. Visual inspection of ring layout
2. Animation smoothness at various refresh rates
3. Readability with all effects disabled
