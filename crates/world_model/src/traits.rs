use crate::context::{WorldContext, WorldSnapshot};
use crate::desktop::{ApplicationInfo, DesktopState, WindowInfo, WorkspaceInfo};
use crate::devices::ConnectedDevice;
use crate::environment::{AmbientConditions, Location, UserEnvironment};
use crate::error::Result;
use crate::event::WorldModelEvent;
use crate::tasks::{ActiveTask, TaskStatus};
use async_trait::async_trait;

#[async_trait]
pub trait WorldModelProvider: Send + Sync {
    async fn current_snapshot(&self) -> Result<WorldSnapshot>;

    async fn refresh(&self) -> Result<()>;

    async fn query_desktop(&self) -> Result<DesktopState>;

    async fn query_environment(&self) -> Result<UserEnvironment>;

    async fn list_devices(&self) -> Result<Vec<ConnectedDevice>>;

    async fn get_device(&self, device_id: &str) -> Result<ConnectedDevice>;

    async fn list_active_tasks(&self) -> Result<Vec<ActiveTask>>;

    async fn get_task(&self, task_id: &str) -> Result<ActiveTask>;

    async fn track_task(&self, task: ActiveTask) -> Result<()>;

    async fn update_task(&self, task_id: &str, status: TaskStatus, progress: f64) -> Result<()>;

    async fn build_context(&self) -> Result<WorldContext>;

    async fn on_event(&self, handler: Box<dyn Fn(WorldModelEvent) + Send + Sync>) -> Result<()>;
}

#[async_trait]
pub trait DesktopMonitor: Send + Sync {
    async fn snapshot(&self) -> Result<DesktopState>;

    async fn list_windows(&self) -> Result<Vec<WindowInfo>>;

    async fn list_applications(&self) -> Result<Vec<ApplicationInfo>>;

    async fn focused_window(&self) -> Option<WindowInfo>;

    async fn active_workspace(&self) -> Option<WorkspaceInfo>;
}

#[async_trait]
pub trait EnvironmentTracker: Send + Sync {
    async fn current_environment(&self) -> Result<UserEnvironment>;

    async fn detect_location(&self) -> Result<Option<Location>>;

    async fn detect_ambient_conditions(&self) -> Result<AmbientConditions>;
}
