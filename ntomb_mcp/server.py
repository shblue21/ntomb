"""ntomb-os-intel MCP Server

Provides process and network connection intelligence for ntomb TUI.
Uses FastMCP for easy tool definition with automatic schema generation.
"""

from mcp.server.fastmcp import FastMCP
import psutil
from typing import Optional

# Initialize MCP server
mcp = FastMCP("ntomb-os-intel", json_response=True)


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


if __name__ == "__main__":
    mcp.run()
