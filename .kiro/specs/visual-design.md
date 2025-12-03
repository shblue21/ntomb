# ntomb Visual Design Specification

## 🎨 1. Color Palette (The Witching Hour Theme)

Use RGB Hex Codes directly instead of generic terminal colors (Red, Green, Blue) to achieve a neon aesthetic.

| Role | Color Name | Hex Code | Usage |
|------|-----------|----------|-------|
| Background | Void Black | #1a1b26 | Terminal background (Tokyo Night theme based) |
| Primary Accent | Neon Purple | #bb9af7 | Main borders, titles, normal data flow |
| Warning/Latency | Pumpkin Orange | #ff9e64 | High latency connections, warning logs |
| Danger/Zombie | Blood Red | #f7768e | Zombie processes, broken connections, error messages |
| Active/Healthy | Toxic Green | #9ece6a | New connections, 'Alive' state, Sparkline graphs |
| Inactive | Bone White | #a9b1d6 | General text, dead nodes (Tombstone) |
| Highlight Background | Deep Indigo | #2f334d | Selected item background color |

## 📐 2. Layout Structure (Ratatui Constraints)

The screen is divided into 3 main sections, with the middle section further split horizontally.

### Layout Hierarchy

1. **Header (Top)**: Fixed height of 8 lines. Space for ASCII Art logo.
2. **Body (Middle)**: Remaining space (Min(0)).
   - **Left Pane (Map)**: 70% width (Percentage(70)). Network topology canvas.
   - **Right Pane (Info)**: 30% width (Percentage(30)). Detailed information.
     - Sub-layout: Vertical split into 3 sections (Details 40%, Traffic Graph 20%, Logs 40%).
3. **Footer (Bottom)**: Fixed height of 3 lines. Status bar and key guide.

## 🧩 3. Component Details (Core Widget Specifications)

### A. The Graveyard (Network Map) - Core Component

**Widget**: Canvas

**Marker**: `Marker::Braille` (Braille mode required). Increases resolution by 2x4 to render smooth curves.

**Drawing Logic**:
- **Nodes**: Display icons and names using text labels (`ctx.print`).
- **Links**: Use `ctx.draw_line`, but instead of drawing straight from (x1, y1) to (x2, y2), add intermediate points using a Bezier Curve algorithm for smooth, organic curves like in the mockup. (Straight lines are acceptable as a starting point if curves are too complex)

**Icons**:
- Center node: ⚰️ (Coffin)
- External nodes: ☁️ (Cloud), �️ (Webo), 👻 (Ghost)

### B. Soul Inspector (Sparkline)

**Widget**: Sparkline

**Data**: Store traffic (Packets/sec) for the last 60 seconds as `Vec<u64>`.

**Style**: Fill with Toxic Green color, with brighter shades for higher data values.

### C. Grimoire (Logs)

**Widget**: List

**Behavior**: Implement 'Auto-scroll' functionality that automatically scrolls down when new logs arrive.

**Prefix**: Change icons based on log level (ℹ️, ⚠️, 🔴).

## ✨ 4. Visual Effects (Wow Points)

This section is critical for impressing judges.

### Neon Gradient Text

Apply gradient coloring to the top banner and bottom status bar, transitioning from purple (left) to orange (right). Use Ratatui's `Line` and `Span` to assign different colors to each character.

### Pulse Animation (Heartbeat)

- Change the `pulse_color` variable every second in the main loop (tick).
- Alternate connection line colors between Purple ↔ Bright Purple to create a living, pulsing data flow effect.

### Zombie Glitch

When a Zombie Process is detected, make the node text flicker by toggling between Visible / Hidden at 0.5-second intervals to create a glitch effect.

## 🎯 Implementation Guidelines

### Color Usage in Ratatui

```rust
use ratatui::style::Color;

// Define color constants
const VOID_BLACK: Color = Color::Rgb(26, 27, 38);
const NEON_PURPLE: Color = Color::Rgb(187, 154, 247);
const PUMPKIN_ORANGE: Color = Color::Rgb(255, 158, 100);
const BLOOD_RED: Color = Color::Rgb(247, 118, 142);
const TOXIC_GREEN: Color = Color::Rgb(158, 206, 106);
const BONE_WHITE: Color = Color::Rgb(169, 177, 214);
const DEEP_INDIGO: Color = Color::Rgb(47, 51, 77);
```

### Canvas with Braille

```rust
use ratatui::widgets::canvas::{Canvas, Context, Line as CanvasLine};
use ratatui::widgets::canvas::Marker;

Canvas::default()
    .marker(Marker::Braille)  // High resolution
    .paint(|ctx| {
        // Draw nodes
        ctx.print(x, y, "⚰️", NEON_PURPLE);
        
        // Draw curved connections
        ctx.draw(&CanvasLine {
            x1, y1, x2, y2,
            color: NEON_PURPLE,
        });
    })
```

### Gradient Text

```rust
// Create gradient from purple to orange
let gradient_text: Vec<Span> = text.chars().enumerate().map(|(i, c)| {
    let ratio = i as f32 / text.len() as f32;
    let r = (187.0 + (255.0 - 187.0) * ratio) as u8;
    let g = (154.0 + (158.0 - 154.0) * ratio) as u8;
    let b = (247.0 - (247.0 - 100.0) * ratio) as u8;
    Span::styled(c.to_string(), Style::default().fg(Color::Rgb(r, g, b)))
}).collect();
```

### Animation State

```rust
struct AppState {
    pulse_phase: f32,  // 0.0 to 1.0
    zombie_blink: bool,
    last_tick: Instant,
}

// In update loop
if now.duration_since(self.last_tick) > Duration::from_millis(500) {
    self.pulse_phase = (self.pulse_phase + 0.1) % 1.0;
    self.zombie_blink = !self.zombie_blink;
    self.last_tick = now;
}
```

## 📝 Notes

- All visual effects should be toggleable for accessibility
- Ensure sufficient contrast for readability
- Test in both light and dark terminal backgrounds
- Provide ASCII fallback for terminals without Unicode support
- Optimize rendering to maintain 60 FPS even with animations
