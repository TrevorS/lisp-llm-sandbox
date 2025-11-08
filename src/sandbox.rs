// ABOUTME: Sandboxed I/O module for the Lisp interpreter
// Provides safe filesystem and network access with capability-based security using cap-std

use crate::config::{FsConfig, NetConfig};
use cap_std::fs::Dir;

#[cfg(test)]
use std::path::PathBuf;

/// Error type for sandbox operations
#[derive(Debug, Clone)]
pub enum SandboxError {
    PathNotAllowed(String),
    FileNotFound(String),
    FileTooLarge(String),
    IoError(String),
    NetworkDisabled,
    AddressNotAllowed(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::PathNotAllowed(path) => {
                write!(f, "Access denied: {} is not in allowed paths", path)
            }
            SandboxError::FileNotFound(path) => {
                write!(f, "File not found: {}", path)
            }
            SandboxError::FileTooLarge(msg) => {
                write!(f, "File too large: {}", msg)
            }
            SandboxError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
            SandboxError::NetworkDisabled => {
                write!(f, "Network I/O is disabled. Use --allow-network to enable.")
            }
            SandboxError::AddressNotAllowed(addr) => {
                write!(f, "Network address not allowed: {}", addr)
            }
        }
    }
}

impl std::error::Error for SandboxError {}

/// Sandbox for safe file and network access
/// Uses capability-based security via cap-std
pub struct Sandbox {
    /// Filesystem sandbox roots
    fs_roots: Vec<Dir>,
    /// Filesystem configuration
    fs_config: FsConfig,
    /// Network configuration
    net_config: NetConfig,
}

impl Sandbox {
    /// Create a new sandbox from configuration
    pub fn new(fs_config: FsConfig, net_config: NetConfig) -> Result<Self, SandboxError> {
        let mut fs_roots = Vec::new();

        // Open all allowed paths as capability directories
        for path in &fs_config.allowed_paths {
            // Create directory if it doesn't exist (for output)
            std::fs::create_dir_all(path).map_err(|e| {
                SandboxError::IoError(format!("Cannot create {}: {}", path.display(), e))
            })?;

            // Open as cap-std Dir (gives us capability-based security)
            let dir = Dir::open_ambient_dir(path, cap_std::ambient_authority()).map_err(|e| {
                SandboxError::IoError(format!("Cannot open {}: {}", path.display(), e))
            })?;

            fs_roots.push(dir);
        }

        Ok(Self {
            fs_roots,
            fs_config,
            net_config,
        })
    }

    // ========================================================================
    // Filesystem Operations
    // ========================================================================

    /// Find which root directory should be used for a path
    /// For reading: tries all roots
    /// For writing: uses first root
    fn find_root_for_path(
        &self,
        user_path: &str,
        write_mode: bool,
    ) -> Result<(&Dir, usize), SandboxError> {
        // cap-std::Dir will automatically reject .. and absolute paths
        // This is secure by construction

        if write_mode {
            // For writes, always use the first root
            if !self.fs_roots.is_empty() {
                return Ok((&self.fs_roots[0], 0));
            }
        } else {
            // For reads, try each root to find the file
            for (idx, root) in self.fs_roots.iter().enumerate() {
                if root.metadata(user_path).is_ok() {
                    return Ok((root, idx));
                }
            }

            // If not found in any root, return first root error
            if !self.fs_roots.is_empty() {
                return Ok((&self.fs_roots[0], 0));
            }
        }

        Err(SandboxError::PathNotAllowed(user_path.to_string()))
    }

    /// Read file contents (safe filesystem access via cap-std)
    pub fn read_file(&self, path: &str) -> Result<String, SandboxError> {
        // Validate path format (no absolute paths, no .. traversals)
        if path.starts_with('/') || path.starts_with("\\") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        if path.contains("..") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        let (root, _) = self.find_root_for_path(path, false)?;

        // cap-std::Dir::read_to_string provides safe access
        root.read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SandboxError::FileNotFound(path.to_string())
            } else {
                SandboxError::IoError(format!("Cannot read {}: {}", path, e))
            }
        })
    }

    /// Write file contents (safe filesystem access via cap-std)
    pub fn write_file(&self, path: &str, contents: &str) -> Result<(), SandboxError> {
        // Validate path format
        if path.starts_with('/') || path.starts_with("\\") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        if path.contains("..") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        // Check size limit before writing
        if contents.len() > self.fs_config.max_file_size {
            return Err(SandboxError::FileTooLarge(format!(
                "{} bytes exceeds limit of {} bytes",
                contents.len(),
                self.fs_config.max_file_size
            )));
        }

        let (root, _) = self.find_root_for_path(path, true)?;

        // cap-std::Dir::write provides safe access
        root.write(path, contents)
            .map_err(|e| SandboxError::IoError(format!("Cannot write {}: {}", path, e)))
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &str) -> Result<bool, SandboxError> {
        // Validate path format
        if path.starts_with('/') || path.starts_with("\\") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        if path.contains("..") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        let (root, _) = self.find_root_for_path(path, false)?;

        match root.metadata(path) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(SandboxError::IoError(format!(
                "Cannot check {}: {}",
                path, e
            ))),
        }
    }

    /// Get file size
    pub fn file_size(&self, path: &str) -> Result<u64, SandboxError> {
        // Validate path format
        if path.starts_with('/') || path.starts_with("\\") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        if path.contains("..") {
            return Err(SandboxError::PathNotAllowed(path.to_string()));
        }

        let (root, _) = self.find_root_for_path(path, false)?;

        root.metadata(path)
            .map(|metadata| metadata.len())
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    SandboxError::FileNotFound(path.to_string())
                } else {
                    SandboxError::IoError(format!("Cannot stat {}: {}", path, e))
                }
            })
    }

    /// List files in a directory
    pub fn list_files(&self, dir: &str) -> Result<Vec<String>, SandboxError> {
        // Validate path format
        if dir.starts_with('/') || dir.starts_with("\\") {
            return Err(SandboxError::PathNotAllowed(dir.to_string()));
        }

        if dir.contains("..") {
            return Err(SandboxError::PathNotAllowed(dir.to_string()));
        }

        let (root, _) = self.find_root_for_path(dir, false)?;

        root.read_dir(dir)
            .map_err(|e| SandboxError::IoError(format!("Cannot list {}: {}", dir, e)))
            .and_then(|entries| {
                entries
                    .map(|entry| {
                        entry
                            .map_err(|e| SandboxError::IoError(e.to_string()))
                            .and_then(|e| {
                                e.file_name()
                                    .to_str()
                                    .map(|s| s.to_string())
                                    .ok_or_else(|| {
                                        SandboxError::IoError(
                                            "Invalid UTF-8 in filename".to_string(),
                                        )
                                    })
                            })
                    })
                    .collect()
            })
    }

    // ========================================================================
    // Network Operations
    // ========================================================================

    /// Check if network is enabled
    /// Reserved for future use in diagnostic/management functions
    #[allow(dead_code)]
    pub fn is_network_enabled(&self) -> bool {
        self.net_config.enabled
    }

    /// Check if an address is allowed
    fn is_address_allowed(&self, address: &str) -> bool {
        if !self.net_config.enabled {
            return false;
        }

        // Empty allowlist = all allowed (if enabled)
        if self.net_config.allowed_addresses.is_empty() {
            return true;
        }

        // Check against allowlist
        self.net_config
            .allowed_addresses
            .iter()
            .any(|allowed| address.contains(allowed))
    }

    /// Perform HTTP GET request with timeout
    /// WARNING: DNS resolution cannot be interrupted by timeouts (ureq limitation).
    /// If DNS lookup hangs, the request will hang indefinitely. Ensure URLs use
    /// responsive DNS servers or use IP addresses directly.
    pub fn http_get(&self, url: &str) -> Result<String, SandboxError> {
        if !self.net_config.enabled {
            return Err(SandboxError::NetworkDisabled);
        }

        if !self.is_address_allowed(url) {
            return Err(SandboxError::AddressNotAllowed(url.to_string()));
        }

        // ureq's timeout applies to socket operations but NOT DNS resolution.
        // This is a platform limitation documented in ureq.
        // See: https://docs.rs/ureq/latest/ureq/#timeouts
        let response = ureq::get(url)
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .map_err(|e| SandboxError::IoError(format!("HTTP GET failed: {}", e)))?;

        response
            .into_string()
            .map_err(|e| SandboxError::IoError(format!("Failed to read response: {}", e)))
    }

    /// Perform HTTP POST request with timeout
    /// WARNING: DNS resolution cannot be interrupted by timeouts (ureq limitation).
    /// If DNS lookup hangs, the request will hang indefinitely. Ensure URLs use
    /// responsive DNS servers or use IP addresses directly.
    pub fn http_post(&self, url: &str, body: &str) -> Result<String, SandboxError> {
        if !self.net_config.enabled {
            return Err(SandboxError::NetworkDisabled);
        }

        if !self.is_address_allowed(url) {
            return Err(SandboxError::AddressNotAllowed(url.to_string()));
        }

        // ureq's timeout applies to socket operations but NOT DNS resolution.
        // This is a platform limitation documented in ureq.
        // See: https://docs.rs/ureq/latest/ureq/#timeouts
        let response = ureq::post(url)
            .set("Content-Type", "text/plain")
            .timeout(std::time::Duration::from_secs(30))
            .send_string(body)
            .map_err(|e| SandboxError::IoError(format!("HTTP POST failed: {}", e)))?;

        response
            .into_string()
            .map_err(|e| SandboxError::IoError(format!("Failed to read response: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;

    fn create_test_sandbox() -> (Sandbox, PathBuf) {
        let test_dir = PathBuf::from("./test_sandbox_temp");
        // Clean up completely from previous run
        let _ = fs::remove_dir_all(&test_dir);
        // Create fresh directory
        fs::create_dir_all(&test_dir).unwrap();

        let fs_config = FsConfig {
            allowed_paths: vec![test_dir.clone()],
            ..Default::default()
        };

        let net_config = NetConfig::default();
        let sandbox = Sandbox::new(fs_config, net_config).unwrap();

        (sandbox, test_dir.clone())
    }

    fn cleanup_test_sandbox(test_dir: &PathBuf) {
        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    #[serial]
    fn test_read_file_success() {
        let (sandbox, test_dir) = create_test_sandbox();

        // Create a test file
        fs::write(test_dir.join("test.txt"), "hello world").unwrap();

        let contents = sandbox.read_file("test.txt").unwrap();
        assert_eq!(contents, "hello world");

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_write_file_success() {
        let (sandbox, test_dir) = create_test_sandbox();

        sandbox.write_file("output.txt", "test data").unwrap();
        let contents = sandbox.read_file("output.txt").unwrap();
        assert_eq!(contents, "test data");

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_file_exists() {
        let (sandbox, test_dir) = create_test_sandbox();

        fs::write(test_dir.join("exists.txt"), "data").unwrap();

        assert!(sandbox.file_exists("exists.txt").unwrap());
        assert!(!sandbox.file_exists("nonexistent.txt").unwrap());

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_path_traversal_rejected() {
        let (sandbox, test_dir) = create_test_sandbox();

        // Attempt path traversal should be rejected
        let result = sandbox.read_file("../../../etc/passwd");
        assert!(matches!(result, Err(SandboxError::PathNotAllowed(_))));

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_absolute_path_rejected() {
        let (sandbox, test_dir) = create_test_sandbox();

        // Absolute paths should be rejected
        let result = sandbox.read_file("/etc/passwd");
        assert!(matches!(result, Err(SandboxError::PathNotAllowed(_))));

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_network_disabled_by_default() {
        let (sandbox, test_dir) = create_test_sandbox();

        let result = sandbox.http_get("https://example.com");
        assert!(matches!(result, Err(SandboxError::NetworkDisabled)));

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_file_size() {
        let (sandbox, test_dir) = create_test_sandbox();

        fs::write(test_dir.join("sized.txt"), "1234567890").unwrap();

        let size = sandbox.file_size("sized.txt").unwrap();
        assert_eq!(size, 10);

        cleanup_test_sandbox(&test_dir);
    }

    #[test]
    #[serial]
    fn test_list_files() {
        let (sandbox, test_dir) = create_test_sandbox();

        fs::write(test_dir.join("file1.txt"), "data1").unwrap();
        fs::write(test_dir.join("file2.txt"), "data2").unwrap();

        let files = sandbox.list_files(".").unwrap();
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));

        cleanup_test_sandbox(&test_dir);
    }
}
