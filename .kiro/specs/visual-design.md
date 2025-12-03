# ntomb Visual Design Specification

## 🎨 1. Color Palette (The Witching Hour Theme)

일반적인 터미널 색상(Red, Green, Blue)을 쓰지 말고, RGB Hex Code를 직접 사용하여 네온 느낌을 내야 합니다.

| 역할 | 색상 이름 | Hex Code | 사용처 |
|------|-----------|----------|--------|
| 배경 | Void Black | #1a1b26 | 전체 터미널 배경 (Tokyo Night 테마 기반) |
| 강조 (Main) | Neon Purple | #bb9af7 | 메인 테두리, 타이틀, 정상적인 데이터 흐름 |
| 경고/지연 | Pumpkin Orange | #ff9e64 | 지연(Latency)이 높은 연결선, 경고 로그 |
| 위험/좀비 | Blood Red | #f7768e | 좀비 프로세스, 끊긴 연결, 에러 메시지 |
| 정상/활성 | Toxic Green | #9ece6a | 새로운 연결, 상태 'Alive', Sparkline 그래프 |
| 비활성 | Bone White | #a9b1d6 | 일반 텍스트, 죽은 노드(Tombstone) |
| 강조 배경 | Deep Indigo | #2f334d | 선택된 항목의 배경색 (Highlight) |

## 📐 2. Layout Structure (Ratatui Constraints)

화면을 크게 3단으로 나누고, 중간 영역을 다시 좌우로 나눕니다.

### Layout Hierarchy

1. **Header (Top)**: 높이 8줄 (고정). ASCII Art 로고가 들어갈 공간.
2. **Body (Middle)**: 나머지 공간 (Min(0)).
   - **Left Pane (Map)**: 너비 70% (Percentage(70)). 네트워크 토폴로지 캔버스.
   - **Right Pane (Info)**: 너비 30% (Percentage(30)). 상세 정보.
     - Sub-layout: 세로로 3등분 (상세정보 40%, 트래픽 그래프 20%, 로그 40%).
3. **Footer (Bottom)**: 높이 3줄 (고정). 상태바 및 키 가이드.

## 🧩 3. Component Details (핵심 위젯 명세)

### A. The Graveyard (Network Map) - 핵심

**Widget**: Canvas

**Marker**: `Marker::Braille` (점자 모드 필수). 해상도를 2x4배 높여 부드러운 곡선을 표현합니다.

**Drawing Logic**:
- **노드(Node)**: 텍스트 라벨 (`ctx.print`)로 아이콘과 이름을 출력.
- **링크(Link)**: `ctx.draw_line`을 사용하되, x1, y1에서 x2, y2로 바로 긋지 말고, 중간 지점을 거치는 베지에 곡선(Bezier Curve) 알고리즘을 살짝 넣으면 목업처럼 유려한 곡선이 나옵니다. (어려우면 직선으로 시작해도 무방)

**Icons**:
- 중앙 노드: ⚰️ (Coffin)
- 외부 노드: ☁️ (Cloud), 🕸️ (Web), 👻 (Ghost)

### B. Soul Inspector (Sparkline)

**Widget**: Sparkline

**Data**: 최근 60초간의 트래픽(Packets/sec)을 `Vec<u64>`로 저장.

**Style**: Toxic Green 색상으로 채우고, 데이터가 높을수록 색이 밝아지게 처리.

### C. Grimoire (Logs)

**Widget**: List

**Behavior**: 새로운 로그가 들어오면 자동으로 스크롤이 아래로 내려가는 'Auto-scroll' 기능 구현.

**Prefix**: 로그 레벨에 따라 아이콘 변경 (ℹ️, ⚠️, 🔴).

## ✨ 4. Visual Effects (Wow Points)

이 부분이 심사위원의 점수를 따는 포인트입니다.

### Neon Gradient Text

상단 배너와 하단 바의 배경색을 단색이 아닌, 왼쪽(보라)에서 오른쪽(주황)으로 변하는 그라데이션 처리를 합니다. (Ratatui의 `Line`과 `Span`을 조합하여 글자마다 색을 다르게 지정).

### Pulse Animation (심장 박동)

- 메인 루프(tick)에서 1초마다 `pulse_color` 변수를 변경합니다.
- 연결선 색상을 Purple ↔ Bright Purple로 번갈아 보여주어, 데이터가 살아 움직이는 느낌을 줍니다.

### Zombie Glitch

좀비 프로세스(Zombie Process)가 감지되면, 해당 노드의 텍스트를 0.5초 간격으로 Visible / Hidden 시켜서 깜빡이는(Flicker) 효과를 줍니다.

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
