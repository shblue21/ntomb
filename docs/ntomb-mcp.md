# ntomb-os-intel MCP Server

ntomb TUI를 위한 OS/네트워크 인텔리전스 MCP 서버입니다.

## 개요

`ntomb-os-intel`은 로컬 머신의 프로세스 및 네트워크 연결 정보를 Kiro에서 도구처럼 사용할 수 있게 해주는 MCP 서버입니다.

## 설치

```bash
cd ntomb_mcp
pip install -r requirements.txt
```

## 로컬 테스트

```bash
# 직접 실행
python -m ntomb_mcp

# 또는
python ntomb_mcp/server.py
```

## Kiro에서 확인

1. Kiro 재시작 또는 MCP 서버 재연결
2. 채팅에서 `/tools` 입력하여 도구 목록 확인
3. `list_connections`, `list_processes`, `get_suspicious_connections` 도구가 보여야 함

## 제공 도구

### 1. `list_connections`

현재 TCP 연결 목록을 반환합니다.

**파라미터:**
- `state_filter` (optional): 연결 상태 필터 (예: "ESTABLISHED", "LISTEN")
- `pid_filter` (optional): 특정 PID만 필터

**반환 스키마:**
```json
[
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
]
```

### 2. `list_processes`

실행 중인 프로세스 목록을 반환합니다.

**파라미터:**
- `name_filter` (optional): 프로세스 이름 필터
- `with_connections` (optional): 연결 수 포함 여부

**반환 스키마:**
```json
[
  {
    "pid": 1234,
    "name": "nginx",
    "cmdline": "nginx: master process",
    "cpu_percent": 0.5,
    "memory_percent": 1.23,
    "connection_count": 42
  }
]
```

### 3. `get_suspicious_connections`

ntomb 휴리스틱을 적용하여 수상한 연결을 식별합니다.

**파라미터:**
- `min_duration_seconds` (default: 600): 장기 연결 기준 (초)
- `high_port_threshold` (default: 49152): 고포트 기준

**반환 스키마:**
```json
[
  {
    "pid": 5678,
    "process_name": "unknown_app",
    "local_address": "192.168.1.10",
    "local_port": 54321,
    "remote_address": "203.0.113.42",
    "remote_port": 54321,
    "state": "ESTABLISHED",
    "reasons": ["external_high_port"]
  }
]
```

**감지 이유 태그:**
- `high_port_listener`: 고포트에서 LISTEN 중
- `external_high_port`: 외부 IP의 고포트로 연결
- `close_wait_leak`: CLOSE_WAIT 상태 (리소스 누수 가능성)

## Rust Struct 매핑

ntomb TUI에서 사용할 Rust struct 예시:

```rust
#[derive(Debug, Deserialize)]
pub struct Connection {
    pub pid: u32,
    pub process_name: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub proto: String,
}

#[derive(Debug, Deserialize)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    #[serde(default)]
    pub connection_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SuspiciousConnection {
    pub pid: u32,
    pub process_name: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub reasons: Vec<String>,
}
```

## 권한 참고

- 일반 사용자: 자신의 프로세스 연결만 조회 가능
- root/sudo: 모든 프로세스 연결 조회 가능
- 권한 부족 시 빈 배열 반환 (에러 없음)
