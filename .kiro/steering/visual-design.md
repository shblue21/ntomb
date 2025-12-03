---
inclusion: always
---

# ntomb Visual Design Guide

## Domain Overview

ntomb is a **terminal-based network graveyard viewer**. It shows live network connections and processes as if they were spirits, coffins, and tombstones in a necromancer’s map.

This guide defines the **visual rules** for:

- color usage,
- layout structure,
- core widgets,
- animation & “wow” effects,
- accessibility and performance.

The goal is to keep the **necromancer / Halloween theme**, while staying **practical for SREs and security engineers** who may run ntomb for hours.

---

## Design Principles

### 1. Clarity Over Decoration

**Rule:** The user should be able to understand “what is going on” in under 3 seconds.

**What this means:**

- Status colors (alive/zombie/warning) must be **consistent across all widgets**.
- Important metrics (latency, state, direction, bytes/sec) should not be hidden behind heavy decoration.
- ASCII art and theme elements **decorate the edges**, not the core data.

### 2. Status-First Color Encoding

**Rule:** Color always encodes **state first**, theme second.

- 🟢 **Toxic Green** = healthy / alive.
- 🟠 **Pumpkin Orange** = warning / high latency / degraded.
- 🔴 **Blood Red** = error, zombie, broken, or “must investigate”.
- 🔵 / 🟣 **Neon Purple** = neutral primary / normal flow.
- ⚪ **Bone White** = baseline text, inactive items, tombstones.

### 3. Accessible by Default

**Rule:** Fancy effects are optional; **readability and contrast** are mandatory.

- All animations must be toggleable.
- The UI must remain legible without Unicode icons (ASCII fallback).
- Colors must keep enough contrast on dark backgrounds.

---

## Color System (The Witching Hour Theme)

### Palette

Use RGB values instead of terminal default colors.

| Role                | Color Name      | Hex Code | Usage                                                                 |
|---------------------|-----------------|----------|-----------------------------------------------------------------------|
| Background          | Void Black      | `#1a1b26` | Terminal background (Tokyo Night-inspired)                           |
| Primary Accent      | Neon Purple     | `#bb9af7` | Main borders, titles, normal connections & flows                     |
| Warning / Latency   | Pumpkin Orange  | `#ff9e64` | High latency, degraded state, warning logs                           |
| Danger / Zombie     | Blood Red       | `#f7768e` | Zombie processes, broken connections, error messages                 |
| Active / Healthy    | Toxic Green     | `#9ece6a` | Alive state, new connections, sparklines for healthy traffic        |
| Inactive / Neutral  | Bone White      | `#a9b1d6` | General text, inactive/dead nodes, tombstones                        |
| Highlight Background| Deep Indigo     | `#2f334d` | Selected items, focused widgets, hover-like states                   |

### Status Mapping

**Rule:** The same event must always map to the same color.

- **Connection state:**
  - `ESTABLISHED (normal)` → Toxic Green
  - `ESTABLISHED (high latency)` → Pumpkin Orange
  - `CLOSE_WAIT / TIME_WAIT storm` → Pumpkin Orange (performance issue)
  - `ZOMBIE / broken` → Blood Red

- **Process state:**
  - Alive, normal → Toxic Green label
  - Zombie / unreaped → Blood Red + “🧟” icon
  - Inactive / dead → Bone White + “🪦” (tombstone)

- **UI chrome:**
  - Borders, section titles → Neon Purple
  - Selected row background → Deep Indigo
  - Status bar gradient → Purple → Orange

---

## Layout Model

The screen is divided into three major regions, inspired by the mockup.

### High-Level Regions

1. **Header (The Necromancer’s Banner)**  
   - Fixed height: **8 lines**  
   - Contains:
     - ASCII logo
     - Subtitle / tagline
     - Global stats (Total Souls, BPF Radar status, etc.)

2. **Body (Main Work Area)**  
   - Fills remaining vertical space.
   - Horizontally split:
     - **Left 70% – “The Graveyard (Network Topology)”**
       - Braille canvas of nodes & connections.
     - **Right 30% – “Soul Inspector + Grimoire”**
       - Vertically split:
         - Top 40% – Soul Inspector (detail panel)
         - Middle 20% – Traffic history sparkline
         - Bottom 40% – Grimoire (logs & alerts)

3. **Footer (Status Bar)**  
   - Fixed height: **3 lines**  
   - Contains:
     - Neon gradient bar
     - Key bindings
     - Short, contextual hint / status message

### Layout Constraints (Ratatui Terms)

- Top-level layout:
  - `Layout::vertical([Constraint::Length(8), Constraint::Min(0), Constraint::Length(3)])`
- Body layout:
  - `Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])`
- Right pane layout:
  - `Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(20), Constraint::Percentage(40)])`

**Invariants:**

- Header and footer heights are constant across screens.
- Graveyard is always visible when ntomb is running.
- Logs (Grimoire) should always show at least **5 visible lines**.

---

## Core Components

### 1. Header – “The Necromancer’s Terminal”

**Purpose:** Branding + high-level info.

**Content:**

- ASCII art title:
  - e.g. `>>> The Necromancer's Terminal v0.9.0 <<<`
- Tagline:
  - `"Revealing the unseen connections of the undead."`
- Quick stats:
  - `[💀 Total Souls: N]`
  - `[🩸 BPF Radar: ACTIVE|INACTIVE]`

**Visual Rules:**

- Background: Void Black  
- Border (if used): Neon Purple  
- Title text: gradient Purple → Orange (see Gradient Text section)  
- Stats: use Emojis + Bone White text, with status colors for values.

---

### 2. The Graveyard (Network Topology Map)

**Widget:** `Canvas` with `Marker::Braille`

**Coordinate system:**

- Normalize to a 0–100 x 0–100 virtual space.
- Keep ~10% padding on edges for labels and icons.

**Nodes:**

- Central services: ⚰️ (coffin)
- Gateways / load balancers: ☁️ (cloud)
- External / unknown endpoints: 👻 (ghost) or 🌐
- Dead / inactive: 🪦 (tombstone)

**Rules:**

- Node label text uses Bone White by default.
- Node icon color reflects **status**:
  - Healthy → Toxic Green
  - Warning → Pumpkin Orange
  - Zombie → Blood Red
- Selected/hovered node:
  - Text in Neon Purple
  - Optional glow: draw a faint Deep Indigo halo around.

**Edges (Connections):**

- Use `ctx.draw` with line primitives (Bezier or polyline).
- **Default color:** Neon Purple
- **High-latency edge:** Pumpkin Orange + dotted/segmented style (if possible)
- **Zombie / error-prone path:** Blood Red, optionally blinking.

**Rendering Notes:**

- Prefer slightly curved paths instead of straight lines.
- Use Braille marker for smooth visual “tendrils”.
- Avoid drawing more than necessary; too many edges at once should degrade to simplified / aggregated view.

---

### 3. Soul Inspector (Detail Panel)

**Purpose:** Show selected node/connection details.

**Content:**

- Target: `TARGET: ⚰️ kafka-broker-1`
- Process info: PID, PPID, USER
- State: textual + colored status indicator
- Key metrics: current latency, bytes in/out, connection count

**Visual Rules:**

- Panel title: Neon Purple
- Field labels: Bone White
- Important values (state, latency):
  - Healthy → Toxic Green
  - High latency → Pumpkin Orange
  - Zombie / error → Blood Red

---

### 4. Traffic History (Sparkline)

**Widget:** `Sparkline`

**Data:**

- Store traffic samples (e.g. bytes/sec or packets/sec) for the last **60 seconds** as `Vec<u64>`.

**Visual Rules:**

- Foreground: Toxic Green
- Low traffic → lower bar height
- Optional feature: overlay a thin line in Neon Purple for moving average.

**Behavior:**

- Always aligned to the right (latest data at the far right).
- Updates on each tick as new samples arrive.

---

### 5. Grimoire (Logs & Alerts)

**Widget:** `List`

**Content:**

- Timestamp `[HH:MM:SS]`
- Level icon:
  - ℹ️ Info (Bone White)
  - ⚠️ Warning (Pumpkin Orange)
  - 🔴 Critical / Zombie (Blood Red)
- Message text (Bone White, with key parts colorized)

**Behavior Rules:**

- Auto-scroll to bottom when following the tail.
- Allow manual scroll; stop auto-scroll if user scrolls up.
- Keep at least last N entries in memory (N configurable).

---

### 6. Footer (Neon Status Bar)

**Content:**

- Gradient bar: Neon Purple → Pumpkin Orange
- Keybind hints:
  - `F1:Help | TAB:Switch Pane | Drag:Pan Map | +/-:Zoom | X:Exorcise (Kill) | Q:R.I.P`

**Rules:**

- Gradient is purely visual; text must remain readable in Bone White.
- If “Exorcise (Kill)” is implemented, highlight it in Blood Red and ensure it’s clearly **opt-in** and consistent with security-domain rules.

---

## Visual Effects & Animations

These are “wow factors” and must remain **optional**.

### 1. Neon Gradient Text

**Rule:** Gradients are used only on:

- Top banner title,
- Bottom status bar background.

**Implementation Sketch:**

```rust
let chars: Vec<char> = text.chars().collect();
let len = chars.len().max(1) as f32;

let spans: Vec<Span> = chars.into_iter().enumerate().map(|(i, c)| {
    let ratio = i as f32 / (len - 1.0).max(1.0);

    // Interpolate between NEON_PURPLE and PUMPKIN_ORANGE
    let r = (187.0 + (255.0 - 187.0) * ratio) as u8;
    let g = (154.0 + (158.0 - 154.0) * ratio) as u8;
    let b = (247.0 + (100.0 - 247.0) * ratio) as u8;

    Span::styled(
        c.to_string(),
        Style::default().fg(Color::Rgb(r, g, b)),
    )
}).collect();
