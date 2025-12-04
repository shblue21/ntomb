"""Detection rules loader and matcher for ntomb-security-analyst.

Parses suspicious_detection.yaml and applies rules to connections.
"""

import yaml
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field


@dataclass
class DetectionRule:
    """A single detection rule from suspicious_detection.yaml."""
    id: str
    name: str
    description: str
    severity: str  # low, medium, high, critical
    tags: list[str]
    match: dict
    effects: dict
    
    def matches_connection(self, conn: dict) -> tuple[bool, list[str]]:
        """Check if this rule matches a connection.
        
        Returns:
            (matched: bool, reasons: list of matching criteria)
        """
        reasons = []
        
        # State matching
        if 'state' in self.match:
            if conn.get('state', '').upper() == self.match['state'].upper():
                reasons.append(f"state={self.match['state']}")
            else:
                return False, []
        
        if 'state_in' in self.match:
            states = [s.upper() for s in self.match['state_in']]
            if conn.get('state', '').upper() in states:
                reasons.append(f"state in {states}")
            else:
                return False, []
        
        # Port matching
        if 'remote_port_gte' in self.match:
            if conn.get('remote_port', 0) >= self.match['remote_port_gte']:
                reasons.append(f"remote_port >= {self.match['remote_port_gte']}")
            else:
                return False, []
        
        if 'local_port_gte' in self.match:
            if conn.get('local_port', 0) >= self.match['local_port_gte']:
                reasons.append(f"local_port >= {self.match['local_port_gte']}")
            else:
                return False, []
        
        if 'local_port_lte' in self.match:
            if conn.get('local_port', 0) <= self.match['local_port_lte']:
                reasons.append(f"local_port <= {self.match['local_port_lte']}")
            else:
                return False, []
        
        # Direction matching (simplified - check if remote is external)
        if 'direction' in self.match and self.match['direction'] == 'outbound':
            remote = conn.get('remote_address', '')
            if remote and not _is_private_ip(remote):
                reasons.append("direction=outbound (external IP)")
            elif 'exclude_private_ips' not in self.match:
                # Allow if not explicitly excluding private IPs
                pass
        
        return len(reasons) > 0, reasons


@dataclass
class DetectionConfig:
    """Configuration and rules from suspicious_detection.yaml."""
    rules: list[DetectionRule] = field(default_factory=list)
    thresholds: dict = field(default_factory=dict)
    tag_definitions: dict = field(default_factory=dict)
    highlight_styles: dict = field(default_factory=dict)


def load_detection_rules(yaml_path: Optional[Path] = None) -> DetectionConfig:
    """Load detection rules from suspicious_detection.yaml.
    
    Args:
        yaml_path: Path to YAML file. If None, searches common locations.
    
    Returns:
        DetectionConfig with parsed rules.
    """
    if yaml_path is None:
        # Search common locations
        search_paths = [
            Path(".kiro/specs/suspicious_detection.yaml"),
            Path("../.kiro/specs/suspicious_detection.yaml"),
            Path(__file__).parent.parent / ".kiro/specs/suspicious_detection.yaml",
        ]
        for path in search_paths:
            if path.exists():
                yaml_path = path
                break
    
    if yaml_path is None or not yaml_path.exists():
        # Return empty config if file not found
        return DetectionConfig()
    
    with open(yaml_path, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    
    config = DetectionConfig()
    
    # Parse thresholds
    if 'defaults' in data and 'thresholds' in data['defaults']:
        config.thresholds = data['defaults']['thresholds']
    
    # Parse rules
    if 'rules' in data:
        for rule_data in data['rules']:
            rule = DetectionRule(
                id=rule_data.get('id', 'unknown'),
                name=rule_data.get('name', 'Unknown Rule'),
                description=rule_data.get('description', '').strip(),
                severity=rule_data.get('severity', 'low'),
                tags=rule_data.get('tags', []),
                match=rule_data.get('match', {}),
                effects=rule_data.get('effects', {}),
            )
            config.rules.append(rule)
    
    # Parse tag definitions
    if 'tag_definitions' in data:
        config.tag_definitions = data['tag_definitions']
    
    # Parse highlight styles
    if 'highlight_styles' in data:
        config.highlight_styles = data['highlight_styles']
    
    return config


def analyze_connection(conn: dict, config: DetectionConfig) -> dict:
    """Analyze a single connection against all detection rules.
    
    Args:
        conn: Connection dict with keys like pid, state, remote_address, etc.
        config: Detection configuration with rules.
    
    Returns:
        Analysis result with matched rules, severity, and explanation.
    """
    matched_rules = []
    max_severity = "normal"
    all_tags = set()
    all_reasons = []
    
    severity_order = {"normal": 0, "low": 1, "medium": 2, "high": 3, "critical": 4}
    
    for rule in config.rules:
        matched, reasons = rule.matches_connection(conn)
        if matched:
            matched_rules.append({
                "rule_id": rule.id,
                "rule_name": rule.name,
                "severity": rule.severity,
                "reasons": reasons,
            })
            all_tags.update(rule.tags)
            all_reasons.extend(reasons)
            
            if severity_order.get(rule.severity, 0) > severity_order.get(max_severity, 0):
                max_severity = rule.severity
    
    return {
        "connection": conn,
        "is_suspicious": len(matched_rules) > 0,
        "severity": max_severity,
        "matched_rules": matched_rules,
        "tags": list(all_tags),
        "match_reasons": list(set(all_reasons)),
    }


def _is_private_ip(ip: str) -> bool:
    """Check if IP is in private/local range."""
    if not ip:
        return True
    if ip.startswith("127.") or ip.startswith("10."):
        return True
    if ip.startswith("192.168."):
        return True
    if ip.startswith("172."):
        parts = ip.split(".")
        if len(parts) >= 2:
            try:
                second = int(parts[1])
                if 16 <= second <= 31:
                    return True
            except ValueError:
                pass
    if ip == "::1" or ip.startswith("fe80:") or ip == "::":
        return True
    if ip == "0.0.0.0":
        return True
    return False


# Korean explanations for rules
RULE_EXPLANATIONS_KO = {
    "long_lived_connection": "장기 연결: 10분 이상 유지된 ESTABLISHED 연결입니다. C2 채널이나 백도어일 수 있습니다.",
    "high_port_beaconing": "고포트 비콘: 49152 이상의 포트로 반복 연결하는 패턴입니다. C2 비콘 통신일 수 있습니다.",
    "suspicious_external_country": "의심 국가 연결: 예상치 못한 국가의 IP로 연결되었습니다. 데이터 유출 가능성을 확인하세요.",
    "unexpected_listener": "예상치 못한 리스너: 비표준 고포트에서 LISTEN 중입니다. 백도어나 미승인 서비스일 수 있습니다.",
    "many_short_lived_connections": "단기 연결 폭주: 짧은 시간에 많은 연결이 열리고 닫혔습니다. 포트 스캔이나 연결 풀 문제일 수 있습니다.",
    "excessive_close_wait": "CLOSE_WAIT 누적: 소켓이 제대로 닫히지 않고 있습니다. 리소스 누수 문제입니다.",
    "excessive_time_wait": "TIME_WAIT 누적: 연결 풀 고갈이나 SO_REUSEADDR 튜닝이 필요할 수 있습니다.",
    "large_data_transfer": "대용량 전송: 100MB 이상의 데이터가 외부로 전송되었습니다. 데이터 유출 가능성을 확인하세요.",
    "connection_to_tor_exit": "Tor 연결: 알려진 Tor 출구 노드로 연결되었습니다. 익명화 통신일 수 있습니다.",
    "failed_connection_attempts": "연결 실패 반복: 같은 대상으로 연결 시도가 반복 실패하고 있습니다.",
    "privileged_port_binding": "특권 포트 바인딩: 1024 미만 포트에 바인딩되었습니다. root 권한이 필요합니다.",
}


def get_rule_explanation_ko(rule_id: str) -> str:
    """Get Korean explanation for a rule."""
    return RULE_EXPLANATIONS_KO.get(rule_id, f"규칙 '{rule_id}'에 매칭되었습니다.")


def generate_investigation_steps(analysis: dict) -> list[str]:
    """Generate recommended investigation steps based on analysis.
    
    Args:
        analysis: Result from analyze_connection()
    
    Returns:
        List of recommended investigation steps in Korean.
    """
    steps = []
    conn = analysis.get("connection", {})
    tags = set(analysis.get("tags", []))
    
    # Common first step
    if conn.get("pid"):
        steps.append(f"1. 프로세스 확인: `ps -p {conn['pid']} -o pid,ppid,user,cmd`")
    
    # Tag-specific steps
    if "beacon" in tags or "c2" in tags:
        steps.append("2. 연결 빈도 분석: 주기적인 패턴이 있는지 확인")
        steps.append("3. 원격 IP 평판 조회: VirusTotal, AbuseIPDB 등에서 확인")
        steps.append("4. 프로세스 바이너리 해시 확인: `sha256sum /proc/<pid>/exe`")
    
    if "exfiltration" in tags:
        steps.append("2. 전송 데이터량 모니터링: `nethogs` 또는 `iftop` 사용")
        steps.append("3. 프로세스가 접근한 파일 확인: `lsof -p <pid>`")
    
    if "resource_leak" in tags or "performance" in tags:
        steps.append("2. 소켓 상태 확인: `ss -s` 또는 `netstat -s`")
        steps.append("3. 애플리케이션 로그 확인")
        steps.append("4. 연결 종료 로직 코드 리뷰")
    
    if "listener" in tags or "backdoor" in tags:
        steps.append("2. 리스닝 포트 확인: `ss -tlnp`")
        steps.append("3. 해당 포트가 의도된 서비스인지 확인")
        steps.append("4. 방화벽 규칙 검토")
    
    if not steps:
        steps.append("1. 연결 상태 지속 모니터링")
        steps.append("2. 관련 프로세스 로그 확인")
    
    return steps
