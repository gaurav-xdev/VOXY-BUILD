use voxy_shared::types::Rect;

#[derive(Debug, Clone)]
pub struct DesktopState {
    pub windows: Vec<WindowInfo>,
    pub active_window_id: Option<String>,
    pub workspaces: Vec<WorkspaceInfo>,
    pub focused_app: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub application_id: String,
    pub application_name: String,
    pub bounds: Option<Rect>,
    pub is_focused: bool,
    pub is_minimized: bool,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub id: String,
    pub name: String,
    pub process_id: Option<u32>,
    pub bundle_id: Option<String>,
    pub is_running: bool,
    pub window_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_state_creation() {
        let state = DesktopState {
            windows: vec![],
            active_window_id: None,
            workspaces: vec![],
            focused_app: None,
        };
        assert!(state.windows.is_empty());
        assert!(state.active_window_id.is_none());
        assert!(state.workspaces.is_empty());
        assert!(state.focused_app.is_none());
    }

    #[test]
    fn test_window_info_creation_with_bounds() {
        let bounds = Rect::new(100, 200, 800, 600);
        let window = WindowInfo {
            id: "win1".to_string(),
            title: "Test Window".to_string(),
            application_id: "app1".to_string(),
            application_name: "Test App".to_string(),
            bounds: Some(bounds),
            is_focused: true,
            is_minimized: false,
            process_id: Some(1234),
        };
        assert_eq!(window.id, "win1");
        assert_eq!(window.title, "Test Window");
        assert_eq!(window.bounds.unwrap(), bounds);
        assert!(window.is_focused);
        assert!(!window.is_minimized);
    }

    #[test]
    fn test_application_info_creation() {
        let app = ApplicationInfo {
            id: "app1".to_string(),
            name: "VS Code".to_string(),
            process_id: Some(5678),
            bundle_id: Some("com.microsoft.vscode".to_string()),
            is_running: true,
            window_count: 3,
        };
        assert_eq!(app.id, "app1");
        assert_eq!(app.name, "VS Code");
        assert_eq!(app.process_id.unwrap(), 5678);
        assert!(app.is_running);
        assert_eq!(app.window_count, 3);
    }

    #[test]
    fn test_workspace_info_creation() {
        let ws = WorkspaceInfo {
            id: "ws1".to_string(),
            name: "Main".to_string(),
            is_active: true,
        };
        assert_eq!(ws.id, "ws1");
        assert_eq!(ws.name, "Main");
        assert!(ws.is_active);
    }
}
