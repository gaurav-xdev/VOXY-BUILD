//! Machine-readable capability manifests for runtimes, providers, and plugins.

pub mod error;
pub mod manifest;
pub mod registry;

pub use error::{ManifestError, Result};
pub use manifest::{CapabilityManifest, ManifestDependency, ManifestPermission};
pub use registry::ManifestRegistry;
