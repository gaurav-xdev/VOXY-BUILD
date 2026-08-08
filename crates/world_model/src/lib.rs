pub mod activity;
pub mod bridge;
pub mod config;
pub mod context;
pub mod desktop;
pub mod devices;
pub mod emitter;
pub mod environment;
pub mod error;
pub use activity::{ActivityClassification, ActivityClassifier, ActivityType};
pub mod event;
pub mod tasks;
pub mod traits;
pub mod watcher;

pub use bridge::{DesktopContext, DesktopEventBridge};
pub use config::WorldModelConfig;
pub use context::{WorldContext, WorldSnapshot};
pub use desktop::{ApplicationInfo, DesktopState, WindowInfo, WorkspaceInfo};
pub use devices::ConnectedDevice;
pub use emitter::{ContextEmitter, DesktopContextUpdate};
pub use environment::{AmbientConditions, Location, UserEnvironment};
pub use error::{Result, WorldModelError};
pub use event::WorldModelEvent;
pub use tasks::{ActiveTask, TaskStatus};
pub use traits::{DesktopMonitor, EnvironmentTracker, WorldModelProvider};
pub use watcher::DesktopWatcher;

pub mod prelude {
    pub use crate::activity::{ActivityClassification, ActivityClassifier, ActivityType};
    pub use crate::bridge::{DesktopContext, DesktopEventBridge};
    pub use crate::config::WorldModelConfig;
    pub use crate::context::{WorldContext, WorldSnapshot};
    pub use crate::desktop::{ApplicationInfo, DesktopState, WindowInfo, WorkspaceInfo};
    pub use crate::devices::ConnectedDevice;
    pub use crate::emitter::{ContextEmitter, DesktopContextUpdate};
    pub use crate::environment::{AmbientConditions, Location, UserEnvironment};
    pub use crate::error::{Result, WorldModelError};
    pub use crate::event::WorldModelEvent;
    pub use crate::tasks::{ActiveTask, TaskStatus};
    pub use crate::traits::{DesktopMonitor, EnvironmentTracker, WorldModelProvider};
    pub use crate::watcher::DesktopWatcher;
}
