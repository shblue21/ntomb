// procfs module - Linux process mapping via /proc filesystem
// Read-only operations following ntomb security-domain guidelines
// Maps network connections to their owning processes using socket inodes

use crate::net::Connection;
use std::io;

#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use tracing::{debug, warn};

/// Map process information to Connections using /proc on Linux
/// No-op on non-Linux systems
///
/// This function reads /proc/<pid>/fd/* to find socket inodes and maps them
/// to connections. It gracefully handles permission errors and continues
/// operation without the affected process information.
///
/// # Arguments
/// * `conns` - Mutable slice of connections to populate with process info
///
/// # Returns
/// * `Ok(())` on success or when running on non-Linux systems
/// * `Err` only on critical failures (not permission errors)
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn attach_process_info(conns: &mut [Connection]) -> io::Result<()> {
    // Non-Linux systems: no-op
    #[cfg(not(target_os = "linux"))]
    {
        let _ = conns; // Suppress unused warning
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Build a map of socket inode -> (pid, process_name)
        let inode_map = build_inode_pid_map()?;

        // Match connections to processes by inode
        for conn in conns.iter_mut() {
            if let Some(inode) = conn.inode {
                if let Some((pid, name)) = inode_map.get(&inode) {
                    conn.pid = Some(*pid);
                    conn.process_name = Some(name.clone());
                }
            }
        }

        debug!(
            "attach_process_info: Mapped {} connections to processes",
            conns.iter().filter(|c| c.pid.is_some()).count()
        );

        Ok(())
    }
}

/// Extract socket inodes from /proc/<pid>/fd/* and build a map
/// Returns HashMap<inode, (pid, process_name)>
#[cfg(target_os = "linux")]
fn build_inode_pid_map() -> io::Result<HashMap<u64, (i32, String)>> {
    let mut map = HashMap::new();
    let proc_path = Path::new("/proc");

    // Check if /proc exists
    if !proc_path.exists() {
        warn!("/proc filesystem not found, cannot map processes");
        return Ok(map);
    }

    // Iterate over /proc/<pid> directories
    let entries = match fs::read_dir(proc_path) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(error = %e, "Cannot read /proc directory");
            return Ok(map);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process numeric directories (PIDs)
        if let Some(filename) = path.file_name() {
            if let Some(pid_str) = filename.to_str() {
                if let Ok(pid) = pid_str.parse::<i32>() {
                    // Read process name from /proc/<pid>/comm
                    let process_name = read_process_name(pid);

                    // Scan /proc/<pid>/fd/* for socket inodes
                    let fd_path = path.join("fd");
                    match fs::read_dir(&fd_path) {
                        Ok(fd_entries) => {
                            for fd_entry in fd_entries.flatten() {
                                // Read the symlink target
                                if let Ok(link_target) = fs::read_link(fd_entry.path()) {
                                    if let Some(target_str) = link_target.to_str() {
                                        // Socket links look like "socket:[12345]"
                                        if target_str.starts_with("socket:[")
                                            && target_str.ends_with(']')
                                        {
                                            // Extract inode number
                                            let inode_str = &target_str[8..target_str.len() - 1];
                                            if let Ok(inode) = inode_str.parse::<u64>() {
                                                map.insert(inode, (pid, process_name.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                            // Permission denied is expected for processes owned by other users
                            // Log at debug level, not warning, as this is normal
                            debug!(pid = pid, "Permission denied reading /proc/{}/fd", pid);
                        }
                        Err(_) => {
                            // Other errors (process exited, etc.) - silently skip
                        }
                    }
                    // Permission errors are expected and handled gracefully
                    // We simply skip processes we can't read
                }
            }
        }
    }

    debug!("build_inode_pid_map: Found {} socket inodes", map.len());
    Ok(map)
}

/// Read process name from /proc/<pid>/comm
/// Returns "unknown" if the file cannot be read
#[cfg(target_os = "linux")]
fn read_process_name(pid: i32) -> String {
    let comm_path = format!("/proc/{}/comm", pid);
    match fs::read_to_string(&comm_path) {
        Ok(name) => name.trim().to_string(),
        Err(_) => {
            // Permission denied or process exited - use "unknown"
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_process_info_empty() {
        let mut conns = vec![];
        let result = attach_process_info(&mut conns);
        assert!(result.is_ok());
    }

    #[test]
    fn test_attach_process_info_no_inode() {
        let mut conns = vec![Connection {
            local_addr: "127.0.0.1".to_string(),
            local_port: 8080,
            remote_addr: "127.0.0.1".to_string(),
            remote_port: 9090,
            state: crate::net::ConnectionState::Established,
            inode: None,
            pid: None,
            process_name: None,
        }];

        let result = attach_process_info(&mut conns);
        assert!(result.is_ok());
        // Without inode, pid should remain None
        assert!(conns[0].pid.is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_build_inode_pid_map() {
        // This is a smoke test - it should succeed even if the map is empty
        let result = build_inode_pid_map();
        assert!(result.is_ok());
        let map = result.unwrap();
        // We can't assert specific contents, but we can verify it's a valid HashMap
        println!("Found {} socket inodes", map.len());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_read_process_name() {
        // Try to read our own process name
        let pid = std::process::id() as i32;
        let name = read_process_name(pid);
        // Should not be empty
        assert!(!name.is_empty());
        println!("Current process name: {}", name);
    }
}
