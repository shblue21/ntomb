"""ntomb-os-intel MCP Server (Security Analyst Edition)

Provides process and network connection intelligence for ntomb TUI,
plus security analysis tools that apply ntomb's detection rules.

Uses FastMCP for easy tool definition with automatic schema generation.
"""

from mcp.server.fastmcp import FastMCP
import psutil
from typing import Optional
from pathlib import Path

from .detection_rules import (
    load_detection_rules,
    analyze_connection,
    get_rule_explanation_ko,
    generate_investigation_steps,
    DetectionConfig,
)

# Initialize MCP server
mcp = FastMCP("ntomb-os-intel", json_response=True)

# Load detection rules at startup
_detection_config: Optional[DetectionConfig] = None

def get_detection_config() -> DetectionConfig:
    """Lazy-load detection configuration."""
    global _detection_config
    if _detection_config is None:
        _detection_config = load_detection_rules()
    return _detection_config


@mcp.tool()
def list_connections(
    state_filter: Optional[str] = None,
    pid_filter: Optional[int] = None
) -> list[dict]:
    """List active TCP network connections with pid, ports, and state.
    
    Returns connection info useful for ntomb's "undead connection" visualization.
    
    Args:
        state_filter: Optional filter by connection state (e.g., "ESTABLISHED", "LISTEN", "TIME_WAIT")
        pid_filter: Optional filter by process ID
    
    Returns:
        List of connection objects with pid, process_name, addresses, ports, state, and protocol.
    """
    connections = []
    
    try:
        for conn in psutil.net_connections(kind='tcp'):
            # Skip if no PID (kernel connections)
            if conn.pid is None:
                continue
            
            # Apply filters
            if pid_filter and conn.pid != pid_filter:
                continue
            
            state = conn.status
            if state_filter and state.upper() != state_filter.upper():
                continue
            
            # Get process name
            try:
                proc = psutil.Process(conn.pid)
                process_name = proc.name()
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                process_name = "unknown"
            
            # Parse addresses
            local_addr = conn.laddr.ip if conn.laddr else ""
            local_port = conn.laddr.port if conn.laddr else 0
            remote_addr = conn.raddr.ip if conn.raddr else ""
            remote_port = conn.raddr.port if conn.raddr else 0
            
            connections.append({
                "pid": conn.pid,
                "process_name": process_name,
                "local_address": local_addr,
                "local_port": local_port,
                "remote_address": remote_addr,
                "remote_port": remote_port,
                "state": state,
                "proto": "tcp"
            })
    except psutil.AccessDenied:
        # Return empty list if no permission (non-root)
        pass
    
    return connections


@mcp.tool()
def list_processes(
    name_filter: Optional[str] = None,
    with_connections: bool = False
) -> list[dict]:
    """List running processes with optional network connection info.
    
    Args:
        name_filter: Optional substring filter for process name
        with_connections: If True, include connection count for each process
    
    Returns:
        List of process objects with pid, name, cmdline, cpu_percent, memory_percent.
    """
    processes = []
    
    # Get connection counts per PID if requested
    conn_counts = {}
    if with_connections:
        try:
            for conn in psutil.net_connections(kind='tcp'):
                if conn.pid:
                    conn_counts[conn.pid] = conn_counts.get(conn.pid, 0) + 1
        except psutil.AccessDenied:
            pass
    
    for proc in psutil.process_iter(['pid', 'name', 'cmdline', 'cpu_percent', 'memory_percent']):
        try:
            info = proc.info
            name = info.get('name', '')
            
            # Apply name filter
            if name_filter and name_filter.lower() not in name.lower():
                continue
            
            cmdline = info.get('cmdline') or []
            
            process_data = {
                "pid": info['pid'],
                "name": name,
                "cmdline": ' '.join(cmdline)[:200],  # Truncate for safety
                "cpu_percent": info.get('cpu_percent', 0.0),
                "memory_percent": round(info.get('memory_percent', 0.0), 2)
            }
            
            if with_connections:
                process_data["connection_count"] = conn_counts.get(info['pid'], 0)
            
            processes.append(process_data)
            
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    
    return processes


@mcp.tool()
def get_suspicious_connections(
    min_duration_seconds: int = 600,
    high_port_threshold: int = 49152
) -> list[dict]:
    """Identify potentially suspicious network connections.
    
    Applies ntomb's heuristics to flag connections that may warrant investigation.
    
    Args:
        min_duration_seconds: Flag ESTABLISHED connections older than this (default: 10 min)
        high_port_threshold: Ports above this are considered "high ports" (default: 49152)
    
    Returns:
        List of suspicious connections with reason tags.
    """
    suspicious = []
    
    try:
        for conn in psutil.net_connections(kind='tcp'):
            if conn.pid is None:
                continue
            
            reasons = []
            
            # High port listener
            if conn.status == 'LISTEN' and conn.laddr:
                if conn.laddr.port > high_port_threshold:
                    reasons.append("high_port_listener")
            
            # External connection on high port
            if conn.status == 'ESTABLISHED' and conn.raddr:
                if conn.raddr.port > high_port_threshold:
                    # Check if remote is not private IP
                    remote_ip = conn.raddr.ip
                    if not _is_private_ip(remote_ip):
                        reasons.append("external_high_port")
            
            # CLOSE_WAIT accumulation (potential resource leak)
            if conn.status == 'CLOSE_WAIT':
                reasons.append("close_wait_leak")
            
            if reasons:
                try:
                    proc = psutil.Process(conn.pid)
                    process_name = proc.name()
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    process_name = "unknown"
                
                suspicious.append({
                    "pid": conn.pid,
                    "process_name": process_name,
                    "local_address": conn.laddr.ip if conn.laddr else "",
                    "local_port": conn.laddr.port if conn.laddr else 0,
                    "remote_address": conn.raddr.ip if conn.raddr else "",
                    "remote_port": conn.raddr.port if conn.raddr else 0,
                    "state": conn.status,
                    "reasons": reasons
                })
    except psutil.AccessDenied:
        pass
    
    return suspicious


def _is_private_ip(ip: str) -> bool:
    """Check if IP is in private/local range."""
    if ip.startswith("127.") or ip.startswith("10."):
        return True
    if ip.startswith("192.168."):
        return True
    if ip.startswith("172."):
        parts = ip.split(".")
        if len(parts) >= 2:
            second = int(parts[1])
            if 16 <= second <= 31:
                return True
    if ip == "::1" or ip.startswith("fe80:"):
        return True
    return False


# =============================================================================
# Security Analyst Tools (ntomb-security-analyst)
# =============================================================================

@mcp.tool()
def analyze_connections() -> dict:
    """Analyze all current connections against ntomb's detection rules.
    
    Applies rules from suspicious_detection.yaml to identify suspicious patterns.
    Returns a security analysis report with categorized findings.
    
    Returns:
        Analysis report with summary, findings by severity, and recommendations.
    """
    config = get_detection_config()
    connections = list_connections()
    
    findings = {
        "critical": [],
        "high": [],
        "medium": [],
        "low": [],
        "normal": [],
    }
    
    all_tags = set()
    
    for conn in connections:
        analysis = analyze_connection(conn, config)
        severity = analysis.get("severity", "normal")
        
        if analysis.get("is_suspicious"):
            finding = {
                "connection": {
                    "pid": conn.get("pid"),
                    "process_name": conn.get("process_name"),
                    "remote": f"{conn.get('remote_address')}:{conn.get('remote_port')}",
                    "local": f"{conn.get('local_address')}:{conn.get('local_port')}",
                    "state": conn.get("state"),
                },
                "matched_rules": [r["rule_name"] for r in analysis.get("matched_rules", [])],
                "tags": analysis.get("tags", []),
            }
            findings[severity].append(finding)
            all_tags.update(analysis.get("tags", []))
        else:
            findings["normal"].append(conn)
    
    # Generate summary
    suspicious_count = sum(len(findings[s]) for s in ["critical", "high", "medium", "low"])
    
    return {
        "summary": {
            "total_connections": len(connections),
            "suspicious_count": suspicious_count,
            "by_severity": {
                "critical": len(findings["critical"]),
                "high": len(findings["high"]),
                "medium": len(findings["medium"]),
                "low": len(findings["low"]),
            },
            "detected_tags": list(all_tags),
        },
        "findings": {
            "critical": findings["critical"],
            "high": findings["high"],
            "medium": findings["medium"],
            "low": findings["low"],
        },
        "rules_loaded": len(config.rules),
    }


@mcp.tool()
def explain_connection(
    pid: Optional[int] = None,
    remote_address: Optional[str] = None,
    remote_port: Optional[int] = None
) -> dict:
    """Explain why a specific connection is flagged as suspicious.
    
    Provides detailed Korean explanation of matched rules and investigation steps.
    
    Args:
        pid: Process ID of the connection
        remote_address: Remote IP address
        remote_port: Remote port number
    
    Returns:
        Detailed explanation with matched rules, Korean description, and next steps.
    """
    config = get_detection_config()
    connections = list_connections(pid_filter=pid)
    
    # Find matching connection
    target_conn = None
    for conn in connections:
        if remote_address and conn.get("remote_address") != remote_address:
            continue
        if remote_port and conn.get("remote_port") != remote_port:
            continue
        target_conn = conn
        break
    
    if not target_conn:
        return {
            "found": False,
            "message": "연결을 찾을 수 없습니다. PID, 원격 주소, 포트를 확인해주세요.",
        }
    
    analysis = analyze_connection(target_conn, config)
    
    # Generate Korean explanations
    explanations = []
    for rule in analysis.get("matched_rules", []):
        rule_id = rule.get("rule_id", "")
        explanation = get_rule_explanation_ko(rule_id)
        explanations.append({
            "rule": rule.get("rule_name"),
            "severity": rule.get("severity"),
            "explanation_ko": explanation,
            "match_reasons": rule.get("reasons", []),
        })
    
    # Generate investigation steps
    investigation_steps = generate_investigation_steps(analysis)
    
    return {
        "found": True,
        "connection": target_conn,
        "is_suspicious": analysis.get("is_suspicious", False),
        "overall_severity": analysis.get("severity", "normal"),
        "tags": analysis.get("tags", []),
        "explanations": explanations,
        "investigation_steps_ko": investigation_steps,
        "summary_ko": _generate_summary_ko(target_conn, analysis),
    }


def _generate_summary_ko(conn: dict, analysis: dict) -> str:
    """Generate Korean summary of the analysis."""
    if not analysis.get("is_suspicious"):
        return f"이 연결은 정상으로 판단됩니다. ({conn.get('process_name', 'unknown')} → {conn.get('remote_address')}:{conn.get('remote_port')})"
    
    severity = analysis.get("severity", "low")
    severity_ko = {"critical": "심각", "high": "높음", "medium": "중간", "low": "낮음"}.get(severity, "알 수 없음")
    
    rules = [r.get("rule_name", "") for r in analysis.get("matched_rules", [])]
    
    return f"⚠️ 수상한 연결 감지 (위험도: {severity_ko})\n" \
           f"프로세스: {conn.get('process_name', 'unknown')} (PID: {conn.get('pid')})\n" \
           f"대상: {conn.get('remote_address')}:{conn.get('remote_port')}\n" \
           f"매칭된 규칙: {', '.join(rules)}"


@mcp.tool()
def get_detection_rules() -> list[dict]:
    """Get all detection rules from suspicious_detection.yaml.
    
    Returns the complete list of rules with their descriptions and match criteria.
    Useful for understanding what patterns ntomb considers suspicious.
    
    Returns:
        List of detection rules with id, name, description, severity, and match criteria.
    """
    config = get_detection_config()
    
    rules = []
    for rule in config.rules:
        rules.append({
            "id": rule.id,
            "name": rule.name,
            "description": rule.description,
            "severity": rule.severity,
            "tags": rule.tags,
            "match_criteria": rule.match,
            "explanation_ko": get_rule_explanation_ko(rule.id),
        })
    
    return rules


@mcp.tool()
def compare_baseline(
    baseline_pids: Optional[list[int]] = None,
    baseline_remotes: Optional[list[str]] = None
) -> dict:
    """Compare current connections against a known baseline.
    
    Identifies new connections that weren't in the baseline, useful for
    detecting changes in network behavior.
    
    Args:
        baseline_pids: List of expected PIDs with network connections
        baseline_remotes: List of expected remote addresses (IP:port format)
    
    Returns:
        Comparison report with new/unexpected connections.
    """
    connections = list_connections()
    
    current_pids = set(c.get("pid") for c in connections if c.get("pid"))
    current_remotes = set(
        f"{c.get('remote_address')}:{c.get('remote_port')}"
        for c in connections
        if c.get("remote_address") and c.get("remote_address") != "0.0.0.0"
    )
    
    baseline_pids_set = set(baseline_pids or [])
    baseline_remotes_set = set(baseline_remotes or [])
    
    new_pids = current_pids - baseline_pids_set if baseline_pids else set()
    new_remotes = current_remotes - baseline_remotes_set if baseline_remotes else set()
    
    # Find connections to new remotes
    new_connections = []
    for conn in connections:
        remote = f"{conn.get('remote_address')}:{conn.get('remote_port')}"
        if remote in new_remotes or conn.get("pid") in new_pids:
            new_connections.append(conn)
    
    return {
        "summary": {
            "total_current_connections": len(connections),
            "new_pids_count": len(new_pids),
            "new_remotes_count": len(new_remotes),
        },
        "new_pids": list(new_pids),
        "new_remote_endpoints": list(new_remotes),
        "new_connections": new_connections,
        "recommendation_ko": _generate_baseline_recommendation_ko(new_pids, new_remotes),
    }


def _generate_baseline_recommendation_ko(new_pids: set, new_remotes: set) -> str:
    """Generate Korean recommendation based on baseline comparison."""
    if not new_pids and not new_remotes:
        return "✅ 베이스라인과 동일합니다. 새로운 연결이 감지되지 않았습니다."
    
    parts = []
    if new_pids:
        parts.append(f"새로운 프로세스 {len(new_pids)}개가 네트워크 연결을 생성했습니다.")
    if new_remotes:
        parts.append(f"새로운 원격 엔드포인트 {len(new_remotes)}개가 감지되었습니다.")
    
    parts.append("이 연결들이 예상된 것인지 확인해주세요.")
    
    return "⚠️ " + " ".join(parts)


@mcp.tool()
def suggest_investigation(pid: int) -> dict:
    """Suggest investigation steps for a specific process.
    
    Analyzes all connections for a process and provides tailored
    investigation recommendations in Korean.
    
    Args:
        pid: Process ID to investigate
    
    Returns:
        Investigation guide with process info, connection analysis, and recommended steps.
    """
    config = get_detection_config()
    
    # Get process info
    try:
        proc = psutil.Process(pid)
        process_info = {
            "pid": pid,
            "name": proc.name(),
            "cmdline": ' '.join(proc.cmdline())[:200],
            "username": proc.username(),
            "create_time": proc.create_time(),
            "status": proc.status(),
        }
    except (psutil.NoSuchProcess, psutil.AccessDenied) as e:
        return {
            "found": False,
            "message": f"프로세스 {pid}를 찾을 수 없거나 접근 권한이 없습니다: {e}",
        }
    
    # Get connections for this process
    connections = list_connections(pid_filter=pid)
    
    # Analyze each connection
    analyses = []
    max_severity = "normal"
    all_tags = set()
    severity_order = {"normal": 0, "low": 1, "medium": 2, "high": 3, "critical": 4}
    
    for conn in connections:
        analysis = analyze_connection(conn, config)
        analyses.append(analysis)
        
        if analysis.get("is_suspicious"):
            all_tags.update(analysis.get("tags", []))
            if severity_order.get(analysis.get("severity", "normal"), 0) > severity_order.get(max_severity, 0):
                max_severity = analysis.get("severity", "normal")
    
    # Generate investigation steps
    investigation_steps = [
        f"1. 프로세스 상세 확인: `ps -p {pid} -o pid,ppid,user,stat,start,cmd`",
        f"2. 열린 파일 확인: `lsof -p {pid}`",
        f"3. 네트워크 연결 확인: `ss -tunap | grep {pid}`",
    ]
    
    if "beacon" in all_tags or "c2" in all_tags:
        investigation_steps.extend([
            "4. 연결 패턴 분석: 주기적인 연결 시도가 있는지 확인",
            "5. 원격 IP 평판 조회: VirusTotal, AbuseIPDB 등",
            f"6. 바이너리 해시 확인: `sha256sum /proc/{pid}/exe`",
        ])
    
    if "exfiltration" in all_tags:
        investigation_steps.extend([
            "4. 트래픽 모니터링: `nethogs` 또는 `tcpdump`로 데이터 흐름 확인",
            "5. 최근 접근 파일 확인: `find /proc/{pid}/fd -type l -exec readlink {} \\;`",
        ])
    
    if "resource_leak" in all_tags:
        investigation_steps.extend([
            "4. 소켓 상태 통계: `ss -s`",
            "5. 애플리케이션 로그 확인",
            "6. 메모리/FD 사용량 모니터링",
        ])
    
    return {
        "found": True,
        "process": process_info,
        "connection_count": len(connections),
        "suspicious_count": sum(1 for a in analyses if a.get("is_suspicious")),
        "overall_severity": max_severity,
        "detected_tags": list(all_tags),
        "connections_summary": [
            {
                "remote": f"{c.get('remote_address')}:{c.get('remote_port')}",
                "state": c.get("state"),
                "suspicious": analyses[i].get("is_suspicious", False),
            }
            for i, c in enumerate(connections)
        ],
        "investigation_steps_ko": investigation_steps,
        "summary_ko": _generate_process_summary_ko(process_info, analyses, all_tags),
    }


def _generate_process_summary_ko(process_info: dict, analyses: list, tags: set) -> str:
    """Generate Korean summary for process investigation."""
    suspicious_count = sum(1 for a in analyses if a.get("is_suspicious"))
    total = len(analyses)
    
    if suspicious_count == 0:
        return f"✅ 프로세스 '{process_info.get('name')}' (PID: {process_info.get('pid')})의 " \
               f"{total}개 연결 중 수상한 패턴이 감지되지 않았습니다."
    
    return f"⚠️ 프로세스 '{process_info.get('name')}' (PID: {process_info.get('pid')})에서 " \
           f"{total}개 연결 중 {suspicious_count}개가 수상한 패턴을 보입니다.\n" \
           f"감지된 태그: {', '.join(tags)}"


# =============================================================================
# Development Assistant Tools (ntomb-dev-assistant)
# =============================================================================

@mcp.tool()
def get_network_map_schema() -> dict:
    """Get the network_map.yaml schema for ntomb development.
    
    Returns the complete schema definition including node types, edge types,
    connection states, and layout rules. Useful for generating Rust structs
    or understanding the data model.
    
    Returns:
        Schema definition with node_types, edge_types, connection_states, and layout rules.
    """
    import yaml
    
    yaml_path = None
    search_paths = [
        Path(".kiro/specs/network_map.yaml"),
        Path("../.kiro/specs/network_map.yaml"),
        Path(__file__).parent.parent / ".kiro/specs/network_map.yaml",
    ]
    
    for path in search_paths:
        if path.exists():
            yaml_path = path
            break
    
    if yaml_path is None or not yaml_path.exists():
        return {
            "found": False,
            "message": "network_map.yaml을 찾을 수 없습니다.",
        }
    
    with open(yaml_path, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    
    # Generate Rust struct suggestions
    rust_structs = _generate_rust_structs_from_schema(data)
    
    return {
        "found": True,
        "schema": data,
        "rust_struct_suggestions": rust_structs,
        "summary": {
            "node_types": list(data.get("node_types", {}).keys()),
            "edge_types": list(data.get("edge_types", {}).keys()),
            "connection_states": list(data.get("connection_states", {}).keys()),
            "views": list(data.get("views", {}).keys()),
        },
    }


def _generate_rust_structs_from_schema(schema: dict) -> list[str]:
    """Generate Rust struct suggestions from network_map schema."""
    structs = []
    
    # Generate node type structs
    for node_name, node_def in schema.get("node_types", {}).items():
        fields = node_def.get("fields", [])
        struct_name = "".join(word.capitalize() for word in node_name.split("_"))
        
        rust_fields = []
        for field in fields:
            field_name = field.get("name", "unknown")
            field_type = _yaml_type_to_rust(field.get("type", "string"), field.get("required", True))
            rust_fields.append(f"    pub {field_name}: {field_type},")
        
        struct_code = f"""#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {struct_name} {{
{chr(10).join(rust_fields)}
}}"""
        structs.append(struct_code)
    
    return structs


def _yaml_type_to_rust(yaml_type: str, required: bool) -> str:
    """Convert YAML type to Rust type."""
    type_map = {
        "u32": "u32",
        "u16": "u16",
        "u64": "u64",
        "usize": "usize",
        "string": "String",
        "bool": "bool",
        "list<string>": "Vec<String>",
    }
    
    rust_type = type_map.get(yaml_type, "String")
    
    if not required:
        return f"Option<{rust_type}>"
    return rust_type


@mcp.tool()
def validate_rule_coverage() -> dict:
    """Validate that detection rules are properly covered in code.
    
    Checks if each rule in suspicious_detection.yaml has corresponding
    implementation hints and identifies potential gaps.
    
    Returns:
        Coverage report with implemented rules, missing implementations, and suggestions.
    """
    config = get_detection_config()
    
    # Rules that have Korean explanations (considered "documented")
    from .detection_rules import RULE_EXPLANATIONS_KO
    
    documented_rules = set(RULE_EXPLANATIONS_KO.keys())
    all_rules = {rule.id for rule in config.rules}
    
    # Check coverage
    covered = all_rules & documented_rules
    undocumented = all_rules - documented_rules
    
    # Analyze rule complexity
    rule_analysis = []
    for rule in config.rules:
        complexity = "simple"
        match_criteria = rule.match
        
        if len(match_criteria) > 3:
            complexity = "complex"
        elif any(k.endswith("_gte") or k.endswith("_lte") for k in match_criteria):
            complexity = "medium"
        
        rule_analysis.append({
            "id": rule.id,
            "name": rule.name,
            "severity": rule.severity,
            "complexity": complexity,
            "match_criteria_count": len(match_criteria),
            "has_korean_explanation": rule.id in documented_rules,
            "tags": rule.tags,
        })
    
    # Generate suggestions for undocumented rules
    suggestions = []
    for rule_id in undocumented:
        rule = next((r for r in config.rules if r.id == rule_id), None)
        if rule:
            suggestions.append({
                "rule_id": rule_id,
                "suggestion": f"RULE_EXPLANATIONS_KO에 '{rule_id}' 설명 추가 필요",
                "template": f'"{rule_id}": "{rule.name}: [한국어 설명 작성]",',
            })
    
    return {
        "summary": {
            "total_rules": len(all_rules),
            "documented_rules": len(covered),
            "undocumented_rules": len(undocumented),
            "coverage_percent": round(len(covered) / len(all_rules) * 100, 1) if all_rules else 0,
        },
        "rules": rule_analysis,
        "undocumented": list(undocumented),
        "suggestions": suggestions,
        "recommendation_ko": _generate_coverage_recommendation_ko(covered, undocumented),
    }


def _generate_coverage_recommendation_ko(covered: set, undocumented: set) -> str:
    """Generate Korean recommendation for rule coverage."""
    if not undocumented:
        return "✅ 모든 규칙이 문서화되어 있습니다."
    
    return f"⚠️ {len(undocumented)}개 규칙에 한국어 설명이 없습니다.\n" \
           f"detection_rules.py의 RULE_EXPLANATIONS_KO에 추가해주세요:\n" \
           f"- {', '.join(list(undocumented)[:5])}" + \
           (f" 외 {len(undocumented) - 5}개" if len(undocumented) > 5 else "")


@mcp.tool()
def suggest_new_rule(
    pattern_description: str,
    observed_connections: Optional[list[dict]] = None
) -> dict:
    """Suggest a new detection rule based on observed patterns.
    
    Analyzes the description and optional connection data to generate
    a new rule definition in suspicious_detection.yaml format.
    
    Args:
        pattern_description: Description of the suspicious pattern in Korean or English
        observed_connections: Optional list of example connections that exhibit the pattern
    
    Returns:
        Suggested rule definition with YAML format and implementation hints.
    """
    # Analyze pattern description for keywords
    keywords = {
        "beacon": ["beacon", "비콘", "주기적", "periodic", "interval"],
        "exfiltration": ["exfil", "유출", "대용량", "large", "transfer"],
        "backdoor": ["backdoor", "백도어", "listener", "리스너", "bind"],
        "scanning": ["scan", "스캔", "probe", "탐색"],
        "c2": ["c2", "command", "control", "명령"],
        "anomaly": ["unusual", "이상", "unexpected", "비정상"],
    }
    
    detected_tags = []
    pattern_lower = pattern_description.lower()
    for tag, words in keywords.items():
        if any(word in pattern_lower for word in words):
            detected_tags.append(tag)
    
    if not detected_tags:
        detected_tags = ["anomaly"]
    
    # Determine severity based on tags
    severity = "medium"
    if "c2" in detected_tags or "exfiltration" in detected_tags:
        severity = "high"
    elif "backdoor" in detected_tags:
        severity = "high"
    elif "scanning" in detected_tags:
        severity = "medium"
    
    # Generate rule ID
    import re
    rule_id = re.sub(r'[^a-z0-9]+', '_', pattern_description.lower()[:30]).strip('_')
    
    # Analyze observed connections if provided
    match_criteria = {}
    if observed_connections:
        states = set(c.get("state", "").upper() for c in observed_connections if c.get("state"))
        if states:
            if len(states) == 1:
                match_criteria["state"] = list(states)[0]
            else:
                match_criteria["state_in"] = list(states)
        
        remote_ports = [c.get("remote_port", 0) for c in observed_connections if c.get("remote_port")]
        if remote_ports:
            min_port = min(remote_ports)
            if min_port > 49152:
                match_criteria["remote_port_gte"] = 49152
            elif min_port > 1024:
                match_criteria["remote_port_gte"] = 1024
    
    # Generate YAML suggestion
    yaml_suggestion = f"""  - id: {rule_id}
    name: "{pattern_description[:50]}"
    description: |
      {pattern_description}
      [자동 생성된 규칙 - 검토 후 수정 필요]
    severity: {severity}
    tags:
{chr(10).join(f'      - {tag}' for tag in detected_tags)}
    match:
{chr(10).join(f'      {k}: {v}' for k, v in match_criteria.items()) if match_criteria else '      # TODO: 매칭 조건 추가'}
    effects:
      add_tag:
{chr(10).join(f'        - {tag}' for tag in detected_tags)}
      highlight_style: {"red_glow" if severity == "high" else "orange_glow" if severity == "medium" else "yellow_glow"}
      icon_hint: {"skull" if severity == "high" else "ghost"}"""
    
    # Generate Korean explanation template
    ko_explanation = f'"{rule_id}": "{pattern_description[:30]}...: [상세 설명 작성]",'
    
    return {
        "suggested_rule": {
            "id": rule_id,
            "name": pattern_description[:50],
            "severity": severity,
            "tags": detected_tags,
            "match_criteria": match_criteria,
        },
        "yaml_format": yaml_suggestion,
        "korean_explanation_template": ko_explanation,
        "implementation_hints": [
            f"1. suspicious_detection.yaml의 rules 섹션에 위 YAML 추가",
            f"2. detection_rules.py의 RULE_EXPLANATIONS_KO에 한국어 설명 추가",
            f"3. 필요시 matches_connection() 메서드에 새 매칭 로직 추가",
        ],
        "recommendation_ko": f"💡 '{pattern_description[:30]}...' 패턴에 대한 규칙을 생성했습니다.\n"
                            f"심각도: {severity}, 태그: {', '.join(detected_tags)}\n"
                            f"위 YAML을 suspicious_detection.yaml에 추가하고 검토해주세요.",
    }


@mcp.tool()
def analyze_spec_consistency() -> dict:
    """Analyze consistency between ntomb specs and implementation.
    
    Checks if the specs (requirements, design, detection rules) are
    consistent with each other and identifies potential gaps.
    
    Returns:
        Consistency report with findings and recommendations.
    """
    import yaml
    
    findings = []
    
    # Load detection rules
    config = get_detection_config()
    detection_rule_ids = {rule.id for rule in config.rules}
    detection_tags = set()
    for rule in config.rules:
        detection_tags.update(rule.tags)
    
    # Load network_map.yaml
    network_map_path = None
    for path in [Path(".kiro/specs/network_map.yaml"), Path("../.kiro/specs/network_map.yaml")]:
        if path.exists():
            network_map_path = path
            break
    
    network_map_states = set()
    if network_map_path:
        with open(network_map_path, 'r', encoding='utf-8') as f:
            network_map = yaml.safe_load(f)
            network_map_states = set(network_map.get("connection_states", {}).keys())
    
    # Check: Detection rules reference valid connection states
    for rule in config.rules:
        match = rule.match
        if "state" in match:
            state = match["state"].upper()
            if state not in network_map_states and network_map_states:
                findings.append({
                    "type": "invalid_state_reference",
                    "severity": "warning",
                    "rule_id": rule.id,
                    "message": f"규칙 '{rule.id}'이 network_map.yaml에 없는 상태 '{state}'를 참조합니다.",
                })
        
        if "state_in" in match:
            for state in match["state_in"]:
                if state.upper() not in network_map_states and network_map_states:
                    findings.append({
                        "type": "invalid_state_reference",
                        "severity": "warning",
                        "rule_id": rule.id,
                        "message": f"규칙 '{rule.id}'이 network_map.yaml에 없는 상태 '{state}'를 참조합니다.",
                    })
    
    # Check: All severity levels are valid
    valid_severities = {"low", "medium", "high", "critical"}
    for rule in config.rules:
        if rule.severity not in valid_severities:
            findings.append({
                "type": "invalid_severity",
                "severity": "error",
                "rule_id": rule.id,
                "message": f"규칙 '{rule.id}'의 심각도 '{rule.severity}'가 유효하지 않습니다.",
            })
    
    # Summary
    error_count = sum(1 for f in findings if f["severity"] == "error")
    warning_count = sum(1 for f in findings if f["severity"] == "warning")
    
    return {
        "summary": {
            "total_findings": len(findings),
            "errors": error_count,
            "warnings": warning_count,
            "detection_rules_count": len(detection_rule_ids),
            "connection_states_count": len(network_map_states),
            "unique_tags_count": len(detection_tags),
        },
        "findings": findings,
        "specs_analyzed": [
            "suspicious_detection.yaml",
            "network_map.yaml" if network_map_path else "(not found)",
        ],
        "recommendation_ko": _generate_consistency_recommendation_ko(findings),
    }


def _generate_consistency_recommendation_ko(findings: list) -> str:
    """Generate Korean recommendation for spec consistency."""
    if not findings:
        return "✅ 스펙 간 일관성 문제가 발견되지 않았습니다."
    
    errors = [f for f in findings if f["severity"] == "error"]
    warnings = [f for f in findings if f["severity"] == "warning"]
    
    parts = []
    if errors:
        parts.append(f"🔴 오류 {len(errors)}개: 즉시 수정 필요")
    if warnings:
        parts.append(f"🟡 경고 {len(warnings)}개: 검토 권장")
    
    return "\n".join(parts)


if __name__ == "__main__":
    mcp.run()
