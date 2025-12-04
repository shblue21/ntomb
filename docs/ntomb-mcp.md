# ntomb-os-intel MCP Server (Security Analyst Edition)

ntomb TUI를 위한 OS/네트워크 인텔리전스 + 보안 분석 MCP 서버입니다.

## 개요

`ntomb-os-intel`은 두 가지 역할을 수행합니다:

1. **OS 인텔리전스**: 프로세스/네트워크 연결 정보 제공
2. **보안 분석가**: `suspicious_detection.yaml` 규칙을 적용해 수상한 연결 탐지 및 설명

## 설치

```bash
cd ntomb_mcp
pip install -r requirements.txt
```

## 로컬 테스트

```bash
python -m ntomb_mcp
```

## Kiro에서 확인

1. Kiro 재시작 또는 MCP 서버 재연결
2. 채팅에서 도구 목록 확인
3. 아래 도구들이 사용 가능해야 함

---

## 제공 도구

### 기본 도구 (OS Intelligence)

#### `list_connections`
현재 TCP 연결 목록을 반환합니다.

```json
{
  "pid": 1234,
  "process_name": "nginx",
  "local_address": "0.0.0.0",
  "local_port": 80,
  "remote_address": "192.168.1.100",
  "remote_port": 54321,
  "state": "ESTABLISHED",
  "proto": "tcp"
}
```

#### `list_processes`
실행 중인 프로세스 목록을 반환합니다.

#### `get_suspicious_connections`
기본 휴리스틱으로 수상한 연결을 식별합니다.

---

### 보안 분석 도구 (Security Analyst)

#### `analyze_connections` ⭐
**모든 연결에 suspicious_detection.yaml 규칙을 적용합니다.**

```json
{
  "summary": {
    "total_connections": 42,
    "suspicious_count": 3,
    "by_severity": {
      "critical": 0,
      "high": 1,
      "medium": 2,
      "low": 0
    },
    "detected_tags": ["beacon", "high_port", "suspicious"]
  },
  "findings": {
    "high": [...],
    "medium": [...]
  }
}
```

#### `explain_connection` ⭐
**특정 연결이 왜 수상한지 한국어로 설명합니다.**

파라미터:
- `pid`: 프로세스 ID
- `remote_address`: 원격 IP
- `remote_port`: 원격 포트

반환 예시:
```json
{
  "found": true,
  "is_suspicious": true,
  "overall_severity": "high",
  "explanations": [
    {
      "rule": "High-Port Beaconing Pattern",
      "severity": "high",
      "explanation_ko": "고포트 비콘: 49152 이상의 포트로 반복 연결하는 패턴입니다. C2 비콘 통신일 수 있습니다."
    }
  ],
  "investigation_steps_ko": [
    "1. 프로세스 확인: `ps -p 1234 -o pid,ppid,user,cmd`",
    "2. 연결 빈도 분석: 주기적인 패턴이 있는지 확인",
    "3. 원격 IP 평판 조회: VirusTotal, AbuseIPDB 등에서 확인"
  ],
  "summary_ko": "⚠️ 수상한 연결 감지 (위험도: 높음)..."
}
```

#### `get_detection_rules`
**suspicious_detection.yaml의 모든 규칙을 반환합니다.**

```json
[
  {
    "id": "high_port_beaconing",
    "name": "High-Port Beaconing Pattern",
    "description": "Detects frequent outbound connections...",
    "severity": "high",
    "tags": ["beacon", "c2", "exfiltration"],
    "explanation_ko": "고포트 비콘: 49152 이상의 포트로 반복 연결하는 패턴입니다..."
  }
]
```

#### `compare_baseline`
**현재 연결을 베이스라인과 비교합니다.**

파라미터:
- `baseline_pids`: 예상되는 PID 목록
- `baseline_remotes`: 예상되는 원격 주소 목록 (IP:port 형식)

```json
{
  "summary": {
    "total_current_connections": 42,
    "new_pids_count": 2,
    "new_remotes_count": 5
  },
  "new_remote_endpoints": ["203.0.113.42:54321", ...],
  "recommendation_ko": "⚠️ 새로운 프로세스 2개가 네트워크 연결을 생성했습니다..."
}
```

#### `suggest_investigation`
**특정 프로세스에 대한 조사 가이드를 제공합니다.**

파라미터:
- `pid`: 조사할 프로세스 ID

```json
{
  "process": {
    "pid": 1234,
    "name": "unknown_app",
    "cmdline": "/usr/bin/unknown_app --daemon"
  },
  "connection_count": 5,
  "suspicious_count": 2,
  "overall_severity": "high",
  "detected_tags": ["beacon", "c2"],
  "investigation_steps_ko": [
    "1. 프로세스 상세 확인: `ps -p 1234 -o pid,ppid,user,stat,start,cmd`",
    "2. 열린 파일 확인: `lsof -p 1234`",
    "3. 네트워크 연결 확인: `ss -tunap | grep 1234`",
    "4. 연결 패턴 분석: 주기적인 연결 시도가 있는지 확인",
    "5. 원격 IP 평판 조회: VirusTotal, AbuseIPDB 등"
  ],
  "summary_ko": "⚠️ 프로세스 'unknown_app' (PID: 1234)에서 5개 연결 중 2개가 수상한 패턴을 보입니다."
}
```

---

## 사용 예시

### Kiro 채팅에서:

```
사용자: "이 서버에서 수상한 연결 있어?"

Kiro: [analyze_connections 호출]
      "3개 연결이 수상한 패턴을 보입니다:
       - PID 4521 (unknown_app) → 203.0.113.42:54321 [high_port_beaconing]
       - PID 1234 (nginx) → 10.0.0.5:8080 [unexpected_listener]
       
       가장 위험한 연결은 PID 4521입니다. 자세히 볼까요?"

사용자: "4521 자세히 분석해줘"

Kiro: [suggest_investigation(pid=4521) 호출]
      "프로세스 'unknown_app'에서 C2 비콘 패턴이 감지되었습니다.
       
       권장 조사 단계:
       1. ps -p 4521 -o pid,ppid,user,cmd
       2. lsof -p 4521
       3. 원격 IP 203.0.113.42를 VirusTotal에서 확인
       ..."
```

---

## Rust Struct 매핑

ntomb TUI에서 사용할 Rust struct:

```rust
#[derive(Debug, Deserialize)]
pub struct AnalysisReport {
    pub summary: AnalysisSummary,
    pub findings: Findings,
    pub rules_loaded: usize,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisSummary {
    pub total_connections: usize,
    pub suspicious_count: usize,
    pub by_severity: HashMap<String, usize>,
    pub detected_tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionExplanation {
    pub found: bool,
    pub is_suspicious: bool,
    pub overall_severity: String,
    pub tags: Vec<String>,
    pub explanations: Vec<RuleExplanation>,
    pub investigation_steps_ko: Vec<String>,
    pub summary_ko: String,
}

#[derive(Debug, Deserialize)]
pub struct RuleExplanation {
    pub rule: String,
    pub severity: String,
    pub explanation_ko: String,
    pub match_reasons: Vec<String>,
}
```

---

## 권한 참고

- 일반 사용자: 자신의 프로세스 연결만 조회 가능
- root/sudo: 모든 프로세스 연결 조회 가능
- 권한 부족 시 빈 배열 반환 (에러 없음)

## 감지 규칙 위치

규칙은 `.kiro/specs/suspicious_detection.yaml`에서 로드됩니다.
규칙을 수정하면 MCP 서버 재시작 후 반영됩니다.
