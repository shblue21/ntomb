---
inclusion: always
---

# ntomb Security Domain Guide

## Domain Overview

ntomb is a **read-only network visualization and inspection tool** for Linux systems. It helps SREs, security engineers, and backend developers understand what network connections a process is making or accepting.

**Key characteristics:**
- **Observational, not interventional:** ntomb reads system state but never modifies it
- **Investigative tool:** Used to spot suspicious patterns, debug network issues, and understand service dependencies
- **Security-aware:** Designed with security use cases in mind, but not a security enforcement tool
- **Context-sensitive:** Understands the difference between local/remote, private/public, expected/unusual connections

## Safety Principles

### 1. Observe, Don't Modify

**Rule:** ntomb only reads data from the operating system and never alters system state.

**What this means:**
- Read from `/proc`, `/sys`, or use read-only commands like `ss`
- Never kill processes, modify firewall rules, close connections, or change network configuration
- Never write to system files or send network packets (except for optional DNS lookups)
- If a feature would require modification, it should be clearly separated and opt-in

**When generating code, Kiro should:**
- Use read-only file operations (`os.ReadFile`, `os.Open`, not `os.WriteFile`)
- Avoid any syscalls that modify state (`kill`, `iptables`, `tc`, etc.)
- If a user explicitly requests a destructive feature, warn about the safety principle and implement it as a clearly separated, opt-in package

### 2. Least Privilege

**Rule:** ntomb should prefer to run without root privileges; root-only features should be clearly documented and optional.

**What this means:**
- Most functionality should work as a regular user viewing their own processes
- Root or elevated privileges may be needed for:
  - Viewing connections of processes owned by other users
  - Reading certain `/proc` entries with restricted permissions
- Document privilege requirements clearly in help text and error messages
- Gracefully degrade when privileges are insufficient (e.g., "Showing only your processes; run with sudo to see all")

**When generating code, Kiro should:**
- Check permissions before attempting privileged operations
- Provide helpful error messages when permissions are insufficient
- Never assume root access; make it optional and explicit

### 3. No Destructive Actions

**Rule:** Kiro should not suggest adding code that blocks, kills, or disrupts anything unless explicitly requested and clearly separated from core ntomb functionality.

**What this means:**
- Core ntomb is purely observational
- If a user asks for "block this connection" or "kill this process", Kiro should:
  - Acknowledge the request
  - Explain that this goes beyond ntomb's core mission
  - Offer to implement it as a separate, opt-in feature with clear warnings
  - Suggest safer alternatives (e.g., "flag for review" instead of "auto-block")

**Example response:**
> "Blocking connections would require modifying firewall rules, which goes beyond ntomb's read-only design. I can add a feature to flag suspicious connections for manual review, or create a separate script that uses iptables if you need automated blocking. Which would you prefer?"

## Network & Security Semantics

### Focus on Visibility and Explanation

ntomb's job is to **show and explain**, not to automatically remediate.

**When generating detection logic or UI annotations, Kiro should:**
- Highlight patterns that warrant investigation
- Explain why something might be suspicious
- Suggest what an operator might check next
- Avoid making absolute judgments ("this IS malware" → "this shows beaconing-like behavior")

**Language guidelines:**
- ✅ "Suspicious", "unusual", "worth investigating", "unexpected"
- ✅ "May indicate", "could suggest", "commonly associated with"
- ❌ "Malicious", "attack", "hacked" (unless there's very strong evidence)
- ❌ Absolute statements without qualifiers

### Context Matters

**Private vs Public IPs:**
- RFC1918 private IPs (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) are less suspicious than random public IPs
- Localhost (127.0.0.1, ::1) connections are usually benign
- Link-local addresses (169.254.0.0/16, fe80::/10) are typically auto-configuration

**Port context:**
- Well-known ports (< 1024) have standard meanings: 22=SSH, 80=HTTP, 443=HTTPS, etc.
- Ephemeral ports (> 32768) are typically client-side ports
- High ports (> 49152) used for listening may be unusual and worth noting

**Connection state context:**
- ESTABLISHED = active data transfer (normal)
- LISTEN = waiting for connections (normal for servers)
- TIME_WAIT = recently closed (normal cleanup, but many may indicate issues)
- CLOSE_WAIT = remote closed, local hasn't (potential resource leak)
- SYN_SENT = connection attempt in progress (many failures may indicate scanning)

**Process context:**
- Web servers connecting to databases: expected
- System daemons connecting to random high ports: unusual
- User processes connecting to cloud storage: depends on context

### Detection Rules Alignment

Detection rules are defined in `.kiro/specs/suspicious_detection.yaml`.

**When implementing classification or highlighting, Kiro should:**
- Reference the detection spec for rule definitions
- Implement rules as described in the spec
- Use the same severity levels (low, medium, high, critical)
- Apply the same tags (beacon, exfiltration, anomaly, etc.)
- Follow the priority and matching logic defined in the spec

**If the spec is unclear or incomplete:**
- Ask for clarification rather than inventing new rules
- Suggest additions to the spec if new patterns are discovered
- Keep implementation aligned with documented behavior

## Data Handling & Privacy

### Sensitive Information

ntomb displays potentially sensitive information:
- IP addresses and ports (may reveal infrastructure topology)
- Domain names (may reveal service dependencies)
- Process names and command lines (may contain credentials or secrets)
- Connection patterns (may reveal business logic or user behavior)

**Guidelines:**
- Treat all displayed data as potentially sensitive
- Design output to be shareable in screenshots/logs (consider future masking features)
- Don't log full sensitive details by default; logs should be useful for debugging without leaking data
- If implementing export features, warn users about data sensitivity

**When generating logging code, Kiro should:**
- Log at appropriate levels (DEBUG for detailed data, INFO for summaries)
- Truncate or redact sensitive fields in logs (e.g., show first 50 chars of cmdline)
- Never log passwords, tokens, or credentials (even if they appear in process cmdlines)

**Example:**
```go
// Good
log.Printf("DEBUG: Found process: pid=%d, name=%s, cmdline=%s", 
           pid, name, truncate(cmdline, 50))

// Avoid
log.Printf("INFO: Process details: %s", fullCmdlineWithSecrets)
```

## "Suspicious" Behavior Heuristics

These heuristics align with `.kiro/specs/suspicious_detection.yaml` and guide what ntomb highlights.

### High-Priority Suspicious Patterns

**Long-lived connections to unusual external endpoints:**
- ESTABLISHED for > 10 minutes to non-standard ports
- May indicate: C2 channels, persistent backdoors, or legitimate long-polling
- Action: Flag for review, show duration prominently

**Repeated high-port outbound connections (beaconing):**
- Frequent connections to same remote IP on high ports (> 49152)
- May indicate: C2 beaconing, data exfiltration
- Action: Highlight with high severity, suggest rate analysis

**Unexpected listeners on high ports:**
- Process listening on non-standard high ports
- May indicate: Backdoors, unauthorized services, misconfigurations
- Action: Flag for investigation, show process details

**Many short-lived or failed connections:**
- Rapid connection cycling, many in TIME_WAIT or SYN_SENT
- May indicate: Port scanning, connection pool issues, aggressive retries
- Action: Show connection rate, suggest checking logs

**Connections to unexpected countries:**
- Outbound connections to countries uncommon for this environment
- May indicate: Data exfiltration, compromised systems
- Action: Show country/ASN, flag for review (but respect legitimate global services)

### Medium-Priority Patterns

**Excessive CLOSE_WAIT connections:**
- Many connections stuck in CLOSE_WAIT
- Indicates: Resource leak, improper socket handling
- Action: Highlight as performance issue, suggest code review

**Large data transfers:**
- Connections with unusually high byte counts
- May indicate: Data exfiltration, backups, legitimate large transfers
- Action: Show transfer size, flag if to unusual destinations

**Unusual protocol mix:**
- Process using unexpected protocols (e.g., HTTP service suddenly using random UDP)
- May indicate: Tunneling, protocol abuse, misconfigurations
- Action: Flag for investigation, show protocol details

### Low-Priority / Informational

**IPv6 usage:**
- Connections using IPv6
- Informational: May be less monitored in some environments
- Action: Tag for visibility, not necessarily suspicious

**Connections to cloud services:**
- Connections to AWS, GCP, Azure, etc.
- Informational: Common and usually legitimate
- Action: Tag for visibility, flag if unexpected for this process

**Privileged port binding:**
- Process binding to ports < 1024
- Informational: Requires elevated privileges
- Action: Tag for security audit, not necessarily suspicious

### Important Caveats

**These are heuristics, not absolute judgments:**
- False positives are expected and acceptable
- Context determines whether a pattern is truly suspicious
- The UI should communicate uncertainty and encourage investigation

**When implementing detection, Kiro should:**
- Use probabilistic language ("may indicate", "commonly associated with")
- Provide context and explanation, not just binary "suspicious" flags
- Allow users to dismiss or override classifications
- Learn from user feedback if possible (future enhancement)

## Tone and Explanations

### Calm, Informative Language

When generating explanations, documentation, or UI text, Kiro should:

**Use calm, professional tone:**
- ✅ "This connection pattern is unusual and may warrant investigation."
- ❌ "ALERT! Your system is under attack!"

**Explain the reasoning:**
- ✅ "Long-lived connections to high ports can indicate C2 channels. Check if this endpoint is expected."
- ❌ "Suspicious connection detected."

**Suggest next steps:**
- ✅ "Consider checking the process's logs, verifying the remote endpoint, or consulting your security team."
- ❌ "You should immediately block this connection."

**Acknowledge uncertainty:**
- ✅ "This pattern is often associated with beaconing, but may also be legitimate polling."
- ❌ "This is definitely malware."

### Avoid Fear-Mongering

**Don't:**
- Use sensational language ("critical threat", "imminent danger")
- Make absolute claims without strong evidence
- Suggest panic or immediate drastic action
- Assume the worst-case scenario

**Do:**
- Present facts and patterns objectively
- Provide context and alternative explanations
- Empower users to make informed decisions
- Maintain a helpful, supportive tone

### Example Explanations

**Good explanation for high-port beaconing:**
> "This process has made 47 connections to 203.0.113.42:54321 in the last 5 minutes. This regular pattern to a high port could indicate C2 beaconing, but may also be legitimate API polling or metrics reporting. Check if this endpoint is expected for this service."

**Good explanation for CLOSE_WAIT buildup:**
> "23 connections are stuck in CLOSE_WAIT state. This typically means the remote side closed the connection, but the application hasn't called close() on its socket. This is a resource leak that can degrade performance over time. Review the application's connection handling code."

**Good explanation for unexpected listener:**
> "This process is listening on port 54123, which is outside the standard port range for this service. This could be a misconfiguration, a development/debug port, or potentially an unauthorized service. Verify this is intentional."

## Summary

When working on ntomb's security and networking features, Kiro should:

- **Observe, don't modify:** Read-only operations only; no destructive actions
- **Least privilege:** Work without root when possible; document privilege requirements
- **Context-aware:** Consider private/public IPs, port meanings, connection states
- **Align with detection spec:** Implement rules as defined in `.kiro/specs/suspicious_detection.yaml`
- **Respect privacy:** Treat displayed data as sensitive; avoid logging secrets
- **Use heuristics wisely:** Flag suspicious patterns, but acknowledge uncertainty
- **Communicate calmly:** Informative, helpful tone; avoid fear-mongering
- **Empower users:** Provide context and next steps, not just alerts

This domain guide ensures ntomb remains a trustworthy, useful tool for security-conscious professionals investigating network behavior on Linux systems.
