# Requirements Document: UI Skeleton

## Introduction

이 문서는 ntomb TUI의 기본 UI 레이아웃과 인터랙션 요구사항을 정의합니다. ntomb는 터미널 기반 네트워크 시각화 도구로, 프로세스 중심의 네트워크 연결 맵을 할로윈 테마(Necromancer/Graveyard)로 표현합니다.

UI Skeleton은 다음 핵심 컴포넌트들의 레이아웃, 렌더링, 인터랙션을 담당합니다:
- Banner (헤더)
- The Graveyard (네트워크 토폴로지 맵)
- Soul Inspector (상세 정보 패널)
- Grimoire (연결 목록)
- Status Bar (상태 바)

모든 UI 요소는 `visual-design.md` 스티어링 문서의 색상 팔레트와 디자인 원칙을 따르며, `security-domain.md`의 읽기 전용 원칙과 차분한 톤을 유지합니다.

## Glossary

- **Graveyard**: 네트워크 토폴로지를 시각화하는 Canvas 위젯 (Braille 마커 사용)
- **HOST**: Graveyard 중앙에 위치한 메인 노드 (현재 호스트 또는 선택된 프로세스)
- **Endpoint**: HOST와 연결된 원격 IP:port 노드
- **Soul Inspector**: 선택된 노드/연결의 상세 정보를 표시하는 패널
- **Grimoire**: 활성 연결 목록을 스크롤 가능하게 표시하는 패널
- **Latency Ring**: HOST 주변의 동심원으로, 레이턴시 버킷별 엔드포인트 위치를 나타냄
- **GraveyardMode**: Host 모드(전체 연결) 또는 Process 모드(특정 프로세스 연결)
- **Pulse Phase**: 애니메이션을 위한 0.0~1.0 사이클 값

## Requirements

### Requirement 1: 메인 레이아웃 구조

**User Story:** 사용자로서, 네트워크 상태를 한눈에 파악할 수 있도록 일관된 레이아웃을 원합니다.

#### Acceptance Criteria

1. WHEN ntomb이 시작되면 THEN 시스템은 3단 수직 레이아웃(Banner 8줄, Body 가변, Status Bar 3줄)을 표시해야 합니다
2. WHEN Body 영역이 렌더링되면 THEN 시스템은 수평으로 분할하여 좌측 65%에 Graveyard, 우측 35%에 상세 패널을 배치해야 합니다
3. WHEN 우측 패널이 렌더링되면 THEN 시스템은 수직으로 분할하여 상단 60%에 Soul Inspector, 하단 40%에 Grimoire를 배치해야 합니다
4. WHEN 터미널 크기가 변경되면 THEN 시스템은 레이아웃 비율을 유지하며 재계산해야 합니다
5. WHEN 최소 터미널 크기(80x24)에서 실행되면 THEN 시스템은 모든 핵심 정보를 표시해야 합니다

### Requirement 2: Banner (헤더) 렌더링

**User Story:** 사용자로서, 애플리케이션 브랜딩과 전역 상태를 헤더에서 확인하고 싶습니다.

#### Acceptance Criteria

1. WHEN Banner가 렌더링되면 THEN 시스템은 ASCII 아트 로고 "NTOMB"를 표시해야 합니다
2. WHEN Banner가 렌더링되면 THEN 시스템은 "The Necromancer's Terminal v0.9.0" 타이틀을 표시해야 합니다
3. WHEN Banner가 렌더링되면 THEN 시스템은 "Revealing the unseen connections of the undead." 태그라인을 표시해야 합니다
4. WHEN Banner가 렌더링되면 THEN 시스템은 전역 통계(Total Souls, BPF Radar 상태)를 표시해야 합니다
5. WHEN Banner가 렌더링되면 THEN 시스템은 Double 테두리와 Neon Purple 색상을 사용해야 합니다

### Requirement 3: Graveyard (네트워크 맵) 렌더링

**User Story:** 사용자로서, 네트워크 연결을 시각적 그래프로 보고 싶습니다.

#### Acceptance Criteria

1. WHEN Graveyard가 렌더링되면 THEN 시스템은 Canvas 위젯과 Braille 마커를 사용해야 합니다
2. WHEN Graveyard가 렌더링되면 THEN 시스템은 중앙(50, 50)에 HOST 관(Coffin)을 표시해야 합니다
3. WHEN 엔드포인트가 존재하면 THEN 시스템은 HOST 주변에 방사형으로 배치해야 합니다
4. WHEN 엔드포인트가 12개를 초과하면 THEN 시스템은 상위 12개만 표시하고 "+N more" 인디케이터를 표시해야 합니다
5. WHEN 연결이 없으면 THEN 시스템은 "The graveyard is quiet..." 또는 "(no active connections for this process)" 메시지를 표시해야 합니다
6. WHEN Graveyard 상단에 요약 라인이 렌더링되면 THEN 시스템은 Endpoints, Listening, Total 카운트를 표시해야 합니다

### Requirement 3.1: 클래식 관(Coffin) 실루엣 - 핵심 요구사항

**User Story:** 사용자로서, Graveyard 중앙에 항상 멋진 관(Coffin) 모양이 완전하게 표시되기를 원합니다. 이것이 ntomb TUI의 핵심 시각 요소입니다.

#### 클래식 관 템플릿 (5줄 Full Version)

```text
  /‾‾‾‾‾‾\
 /  ⚰️    \
/   HOST   \
 \        /
  \______/
```

- 상단/하단이 대칭인 전형적인 관(coffin) 실루엣
- 가운데 줄(HOST)에 실제 호스트/프로세스 이름을 표시
- 관의 윤곽선(슬래시, 백슬래시, 밑줄 등)은 **절대로 깨지거나 잘려 보이면 안 됨**

#### Acceptance Criteria

1. WHEN Graveyard 중앙에 HOST 관이 렌더링되면 THEN 시스템은 위/아래가 대칭인 클래식 관 실루엣을 표시해야 합니다
2. WHEN HOST 관이 렌더링되면 THEN 시스템은 관의 모든 외곽선(/, \, ‾, _)이 완전히 표시되어야 합니다 (일부만 잘려 보이는 상태 금지)
3. WHEN 호스트 이름이 관 내부 폭을 초과하면 THEN 시스템은 이름을 중앙 정렬하고 필요시 "..."로 잘라내야 합니다
4. WHEN 터미널/패널 크기가 5줄 관을 표시하기에 부족하면 THEN 시스템은 3줄 컴팩트 버전으로 graceful degrade 해야 합니다
5. WHEN 3줄 관도 표시할 수 없으면 THEN 시스템은 "[⚰ HOST]" 형태의 단일 라벨로 최종 fallback 해야 합니다
6. WHEN 관이 표시되면 THEN 시스템은 관 주변 1~2 캔버스 유닛 정도의 여백을 확보하여 다른 노드/선이 침범하지 않도록 해야 합니다
7. WHEN 관이 렌더링되면 THEN 시스템은 Neon Purple 색상을 기본으로 사용하고, Overdrive 모드에서는 Pumpkin Orange를 사용해야 합니다

#### 관 변형 (Variants)

**Full 5-Line (기본):**
```text
  /‾‾‾‾‾‾\
 /  ⚰️    \
/   HOST   \
 \        /
  \______/
```

**Compact 3-Line (공간 부족 시):**
```text
 /‾‾‾‾‾‾\
 | ⚰ HOST |
 \______/
```

**Label Only (최소 공간):**
```text
[⚰ HOST]
```

### Requirement 4: 엔드포인트 노드 시각화

**User Story:** 사용자로서, 연결 상태를 아이콘과 색상으로 즉시 파악하고 싶습니다.

#### Acceptance Criteria

1. WHEN 엔드포인트가 ESTABLISHED 상태이면 THEN 시스템은 🎃 아이콘과 Toxic Green 색상을 사용해야 합니다
2. WHEN 엔드포인트가 TIME_WAIT 상태이면 THEN 시스템은 👻 아이콘과 Pumpkin Orange 색상을 사용해야 합니다
3. WHEN 엔드포인트가 CLOSE_WAIT 상태이면 THEN 시스템은 💀 아이콘과 Pumpkin Orange 색상을 사용해야 합니다
4. WHEN 엔드포인트가 SYN_SENT 상태이면 THEN 시스템은 ⏳ 아이콘을 사용해야 합니다
5. WHEN 엔드포인트가 LISTEN 상태이면 THEN 시스템은 👂 아이콘을 사용해야 합니다
6. WHEN 엔드포인트가 기타 상태이면 THEN 시스템은 🌐 아이콘과 Bone White 색상을 사용해야 합니다
7. WHEN 엔드포인트 레이블이 15자를 초과하면 THEN 시스템은 12자로 잘라서 "..."를 추가해야 합니다

### Requirement 5: 연결 엣지 렌더링

**User Story:** 사용자로서, HOST와 엔드포인트 간의 연결을 시각적으로 확인하고 싶습니다.

#### Acceptance Criteria

1. WHEN 연결 엣지가 렌더링되면 THEN 시스템은 HOST 중심에서 각 엔드포인트까지 선을 그려야 합니다
2. WHEN 연결이 ESTABLISHED 상태이면 THEN 시스템은 Toxic Green 색상 선을 사용해야 합니다
3. WHEN 연결이 TIME_WAIT 또는 CLOSE_WAIT 상태이면 THEN 시스템은 Pumpkin Orange 색상 선을 사용해야 합니다
4. WHEN 연결이 SYN_SENT 또는 SYN_RECV 상태이면 THEN 시스템은 Yellow 색상 선을 사용해야 합니다
5. WHEN 연결이 CLOSE 상태이면 THEN 시스템은 Blood Red 색상 선을 사용해야 합니다
6. WHEN 연결이 기타 상태이면 THEN 시스템은 pulse_phase 기반 애니메이션 색상을 사용해야 합니다

### Requirement 6: Soul Inspector 패널

**User Story:** 사용자로서, 선택된 대상의 상세 정보를 확인하고 싶습니다.

#### Acceptance Criteria

1. WHEN Soul Inspector가 렌더링되면 THEN 시스템은 TARGET, PID, PPID, USER, STATE 정보를 표시해야 합니다
2. WHEN Soul Inspector가 렌더링되면 THEN 시스템은 현재 Refresh 간격(ms)을 표시해야 합니다
3. WHEN Refresh 간격이 최근 변경되었으면 THEN 시스템은 해당 값을 강조(Bold, Underline)해야 합니다
4. WHEN Soul Inspector가 렌더링되면 THEN 시스템은 Traffic History Sparkline을 표시해야 합니다
5. WHEN Soul Inspector가 렌더링되면 THEN 시스템은 Open Sockets List를 표시해야 합니다
6. WHEN Soul Inspector가 렌더링되면 THEN 시스템은 Rounded 테두리와 Neon Purple 색상을 사용해야 합니다

### Requirement 7: Grimoire (연결 목록) 패널

**User Story:** 사용자로서, 모든 활성 연결을 스크롤 가능한 목록으로 보고 싶습니다.

#### Acceptance Criteria

1. WHEN Grimoire가 렌더링되면 THEN 시스템은 모든 연결을 번호와 함께 목록으로 표시해야 합니다
2. WHEN 연결이 표시되면 THEN 시스템은 "local:port → remote:port [STATE]" 형식을 사용해야 합니다
3. WHEN 연결이 LISTEN 상태이면 THEN 시스템은 "local:port [LISTEN]" 형식을 사용해야 합니다
4. WHEN 프로세스 정보가 있으면 THEN 시스템은 "[name(pid)]" 태그를 연결 끝에 추가해야 합니다
5. WHEN 연결이 선택되면 THEN 시스템은 Deep Indigo 배경색으로 강조해야 합니다
6. WHEN 연결 상태에 따라 THEN 시스템은 적절한 색상(ESTABLISHED=Toxic Green, LISTEN=Bone White, TIME_WAIT/CLOSE_WAIT=Pumpkin Orange, CLOSE=Blood Red)을 적용해야 합니다
7. WHEN Grimoire가 렌더링되면 THEN 시스템은 Rounded 테두리와 Pumpkin Orange 색상을 사용해야 합니다

### Requirement 8: Status Bar

**User Story:** 사용자로서, 사용 가능한 키보드 단축키와 현재 상태를 확인하고 싶습니다.

#### Acceptance Criteria

1. WHEN Status Bar가 렌더링되면 THEN 시스템은 💀 아이콘과 키보드 힌트를 표시해야 합니다
2. WHEN Host 모드이면 THEN 시스템은 "P:Focus Process" 힌트를 표시해야 합니다
3. WHEN Process 모드이면 THEN 시스템은 "P:Back to Host" 힌트를 표시해야 합니다
4. WHEN Status Bar가 렌더링되면 THEN 시스템은 토글 상태 인디케이터([A:ON/OFF], [H:ON/OFF], [t:ON/OFF])를 표시해야 합니다
5. WHEN 토글이 활성화되면 THEN 시스템은 Toxic Green 색상을 사용해야 합니다
6. WHEN 토글이 비활성화되면 THEN 시스템은 Bone White 색상을 사용해야 합니다
7. WHEN 터미널 너비가 좁으면 THEN 시스템은 우선순위에 따라 힌트를 생략해야 합니다
8. WHEN Status Bar가 렌더링되면 THEN 시스템은 Double 테두리와 Neon Purple 색상을 사용해야 합니다

### Requirement 9: 키보드 인터랙션

**User Story:** 사용자로서, 키보드로 UI를 탐색하고 제어하고 싶습니다.

#### Acceptance Criteria

1. WHEN 사용자가 'q', 'Q', 또는 Esc를 누르면 THEN 시스템은 애플리케이션을 종료해야 합니다
2. WHEN 사용자가 ↑/↓ 화살표를 누르면 THEN 시스템은 연결 선택을 이동해야 합니다
3. WHEN 사용자가 'p' 또는 'P'를 누르면 THEN 시스템은 GraveyardMode를 토글해야 합니다
4. WHEN 사용자가 Tab을 누르면 THEN 시스템은 패널 포커스를 전환해야 합니다
5. WHEN 사용자가 '+' 또는 '='를 누르면 THEN 시스템은 리프레시 속도를 증가해야 합니다
6. WHEN 사용자가 '-' 또는 '_'를 누르면 THEN 시스템은 리프레시 속도를 감소해야 합니다
7. WHEN 사용자가 'a' 또는 'A'를 누르면 THEN 시스템은 애니메이션을 토글해야 합니다
8. WHEN 사용자가 'h' 또는 'H'를 누르면 THEN 시스템은 Kiroween Overdrive 모드를 토글해야 합니다
9. WHEN 사용자가 't' 또는 'T'를 누르면 THEN 시스템은 엔드포인트 레이블을 토글해야 합니다

### Requirement 10: 애니메이션 및 시각 효과

**User Story:** 사용자로서, 네트워크 활동을 동적인 시각 효과로 인지하고 싶습니다.

#### Acceptance Criteria

1. WHEN 애니메이션이 활성화되면 THEN 시스템은 pulse_phase(0.0~1.0)를 100ms마다 업데이트해야 합니다
2. WHEN 애니메이션이 활성화되면 THEN 시스템은 연결 엣지에 색상 펄스 효과를 적용해야 합니다
3. WHEN zombie_blink가 활성화되면 THEN 시스템은 500ms마다 좀비 상태 노드의 가시성을 토글해야 합니다
4. WHEN Traffic History가 업데이트되면 THEN 시스템은 Sparkline에 새 데이터를 추가해야 합니다
5. WHEN 애니메이션이 비활성화되면 THEN 시스템은 정적 렌더링을 유지하며 동일한 정보를 표시해야 합니다

### Requirement 11: 색상 팔레트 준수

**User Story:** 사용자로서, 일관된 할로윈 테마 색상으로 UI를 경험하고 싶습니다.

#### Acceptance Criteria

1. WHEN UI가 렌더링되면 THEN 시스템은 Neon Purple (#bb9af7)을 주요 테두리와 타이틀에 사용해야 합니다
2. WHEN UI가 렌더링되면 THEN 시스템은 Pumpkin Orange (#ff9e64)를 경고 상태와 강조에 사용해야 합니다
3. WHEN UI가 렌더링되면 THEN 시스템은 Blood Red (#f7768e)를 에러 상태에 사용해야 합니다
4. WHEN UI가 렌더링되면 THEN 시스템은 Toxic Green (#9ece6a)를 정상/활성 상태에 사용해야 합니다
5. WHEN UI가 렌더링되면 THEN 시스템은 Bone White (#a9b1d6)를 일반 텍스트와 비활성 상태에 사용해야 합니다
6. WHEN 선택된 항목이 있으면 THEN 시스템은 Deep Indigo (#2f334d)를 배경색으로 사용해야 합니다

