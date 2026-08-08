//! Version and build information.
//!
//! Provides semantic versioning support and build metadata.

use serde::{Deserialize, Serialize};

/// Version information for the VOXY platform.
///
/// Follows semantic versioning (semver).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    major: u32,
    minor: u32,
    patch: u32,
    pre_release: Option<String>,
    build_metadata: Option<String>,
}

impl VersionInfo {
    /// Create a new version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    /// Get the major version.
    pub fn major(&self) -> u32 {
        self.major
    }

    /// Get the minor version.
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// Get the patch version.
    pub fn patch(&self) -> u32 {
        self.patch
    }

    /// Get the pre-release identifier.
    pub fn pre_release(&self) -> Option<&str> {
        self.pre_release.as_deref()
    }

    /// Get the build metadata.
    pub fn build_metadata(&self) -> Option<&str> {
        self.build_metadata.as_deref()
    }

    /// Set the pre-release identifier.
    pub fn with_pre_release(mut self, pre_release: impl Into<String>) -> Self {
        self.pre_release = Some(pre_release.into());
        self
    }

    /// Set the build metadata.
    pub fn with_build_metadata(mut self, build_metadata: impl Into<String>) -> Self {
        self.build_metadata = Some(build_metadata.into());
        self
    }

    /// Check if this is a pre-release version.
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    /// Compare versions ignoring pre-release and build metadata.
    pub fn major_minor_patch(&self) -> (u32, u32, u32) {
        (self.major, self.minor, self.patch)
    }
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(build) = &self.build_metadata {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

/// Build information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    timestamp: String,
    git_commit: String,
    git_branch: String,
    profile: String,
    target: String,
    rust_version: String,
}

impl BuildInfo {
    /// Get the build timestamp.
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// Get the git commit hash.
    pub fn git_commit(&self) -> &str {
        &self.git_commit
    }

    /// Get the git branch.
    pub fn git_branch(&self) -> &str {
        &self.git_branch
    }

    /// Get the build profile.
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Get the build target triple.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Get the Rust compiler version.
    pub fn rust_version(&self) -> &str {
        &self.rust_version
    }
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            timestamp: "unknown".to_string(),
            git_commit: "unknown".to_string(),
            git_branch: "unknown".to_string(),
            profile: "debug".to_string(),
            target: "unknown".to_string(),
            rust_version: "unknown".to_string(),
        }
    }
}

/// Get the current VOXY version.
pub fn version() -> VersionInfo {
    VersionInfo::new(0, 1, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_creation() {
        let version = VersionInfo::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 3);
    }

    #[test]
    fn version_with_prerelease() {
        let version = VersionInfo::new(1, 2, 3).with_pre_release("alpha.1");
        assert_eq!(version.to_string(), "1.2.3-alpha.1");
        assert!(version.is_pre_release());
    }

    #[test]
    fn version_with_build_metadata() {
        let version = VersionInfo::new(1, 2, 3).with_build_metadata("build.123");
        assert_eq!(version.to_string(), "1.2.3+build.123");
        assert_eq!(version.build_metadata(), Some("build.123"));
    }

    #[test]
    fn version_full() {
        let version = VersionInfo::new(1, 2, 3)
            .with_pre_release("alpha.1")
            .with_build_metadata("build.123");
        assert_eq!(version.to_string(), "1.2.3-alpha.1+build.123");
    }

    #[test]
    fn default_version() {
        let version = VersionInfo::default();
        assert_eq!(version.to_string(), "0.1.0");
    }

    #[test]
    fn current_version() {
        let v = version();
        assert_eq!(v.major(), 0);
        assert_eq!(v.minor(), 1);
        assert_eq!(v.patch(), 0);
    }

    #[test]
    fn version_partial_eq() {
        let v1 = VersionInfo::new(1, 2, 3);
        let v2 = VersionInfo::new(1, 2, 3);
        let v3 = VersionInfo::new(1, 2, 4);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }
}
