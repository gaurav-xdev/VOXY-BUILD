use async_trait::async_trait;
use tracing::info;

use crate::error::Result;
use voxy_orchestrator::automation::{
    AutomationBackend, AutomationCapability, ElementInfo, ElementSelector, MouseButton,
    StateVerification, WindowTarget,
};
use voxy_shared::types::Rect;

pub struct OpenClawBackend {
    name: String,
    endpoint: String,
    /// SECURITY: API key stored as Zeroizing to prevent memory exposure on drop.
    api_key: Option<zeroize::Zeroizing<String>>,
    timeout_seconds: u64,
    available: bool,
}

impl OpenClawBackend {
    pub fn new(endpoint: String, api_key: Option<String>, timeout_seconds: u64) -> Self {
        Self {
            name: "openclaw".to_string(),
            endpoint,
            api_key: api_key.map(zeroize::Zeroizing::new),
            timeout_seconds,
            available: false,
        }
    }

    async fn json_rpc_call(
        &self,
        _method: &str,
        _params: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, String> {
        Err(
            "OpenClaw endpoint not available - use enigo/win32 backend for direct execution"
                .to_string(),
        )
    }
}

impl Default for OpenClawBackend {
    fn default() -> Self {
        Self::new("http://127.0.0.1:9876".to_string(), None, 30)
    }
}

#[async_trait]
impl AutomationBackend for OpenClawBackend {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(&self) -> Result<()> {
        info!(
            "OpenClaw backend initializing (endpoint: {})",
            self.endpoint
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        info!("OpenClaw backend shutting down");
        Ok(())
    }

    async fn is_available(&self) -> bool {
        self.available
    }

    async fn click(&self, x: i32, y: i32, button: MouseButton) -> Result<()> {
        let btn = match button {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
            MouseButton::X1 => "x1",
            MouseButton::X2 => "x2",
        };
        let params = serde_json::json!({
            "x": x,
            "y": y,
            "button": btn,
        });
        self.json_rpc_call("click", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw click failed: {}", e)))?;
        Ok(())
    }

    async fn double_click(&self, x: i32, y: i32) -> Result<()> {
        self.click(x, y, MouseButton::Left).await?;
        self.click(x, y, MouseButton::Left).await
    }

    async fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        let params = serde_json::json!({ "x": x, "y": y });
        self.json_rpc_call("move_mouse", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw move failed: {}", e)))?;
        Ok(())
    }

    async fn drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<()> {
        let params = serde_json::json!({
            "from_x": from_x,
            "from_y": from_y,
            "to_x": to_x,
            "to_y": to_y,
        });
        self.json_rpc_call("drag", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw drag failed: {}", e)))?;
        Ok(())
    }

    async fn scroll(&self, x: i32, y: i32, delta_x: i32, delta_y: i32) -> Result<()> {
        let params = serde_json::json!({
            "x": x,
            "y": y,
            "delta_x": delta_x,
            "delta_y": delta_y,
        });
        self.json_rpc_call("scroll", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw scroll failed: {}", e)))?;
        Ok(())
    }

    async fn type_text(&self, text: &str, interval_ms: u64) -> Result<()> {
        let params = serde_json::json!({
            "text": text,
            "interval_ms": interval_ms,
        });
        self.json_rpc_call("type_text", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw type_text failed: {}", e)))?;
        Ok(())
    }

    async fn key_press(&self, key: &str) -> Result<()> {
        let params = serde_json::json!({ "key": key });
        self.json_rpc_call("key_press", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw key_press failed: {}", e)))?;
        Ok(())
    }

    async fn key_combination(&self, keys: &[&str]) -> Result<()> {
        let params = serde_json::json!({ "keys": keys });
        self.json_rpc_call("key_combination", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw key_combination failed: {}", e))
            })?;
        Ok(())
    }

    async fn hold_key(&self, key: &str, duration_ms: u64) -> Result<()> {
        self.key_press(key).await?;
        tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
        self.key_press(key).await
    }

    async fn screenshot(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "x": x,
            "y": y,
            "width": width,
            "height": height,
        });
        let result = self
            .json_rpc_call("screenshot", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw screenshot failed: {}", e)))?;
        let bytes = result
            .as_str()
            .and_then(|s| base64_decode(s))
            .ok_or_else(|| crate::error::action_err("invalid screenshot response"))?;
        Ok(bytes)
    }

    async fn screen_size(&self) -> Result<(u32, u32)> {
        let result = self
            .json_rpc_call("screen_size", serde_json::json!({}))
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw screen_size failed: {}", e)))?;
        let w = result["width"].as_u64().unwrap_or(0) as u32;
        let h = result["height"].as_u64().unwrap_or(0) as u32;
        Ok((w, h))
    }

    async fn get_pixel_color(&self, x: i32, y: i32) -> Result<(u8, u8, u8)> {
        let params = serde_json::json!({ "x": x, "y": y });
        let result = self
            .json_rpc_call("get_pixel_color", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw get_pixel_color failed: {}", e))
            })?;
        let r = result["r"].as_u64().unwrap_or(0) as u8;
        let g = result["g"].as_u64().unwrap_or(0) as u8;
        let b = result["b"].as_u64().unwrap_or(0) as u8;
        Ok((r, g, b))
    }

    async fn get_active_window(&self) -> Result<WindowTarget> {
        let result = self
            .json_rpc_call("get_active_window", serde_json::json!({}))
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw get_active_window failed: {}", e))
            })?;
        serde_json::from_value(result)
            .map_err(|e| crate::error::action_err(format!("invalid window target: {}", e)))
    }

    async fn find_window(&self, title: &str, class: Option<&str>) -> Result<Vec<WindowTarget>> {
        let params = serde_json::json!({ "title": title, "class": class });
        let result = self
            .json_rpc_call("find_window", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw find_window failed: {}", e)))?;
        serde_json::from_value(result)
            .map_err(|e| crate::error::action_err(format!("invalid window targets: {}", e)))
    }

    async fn focus_window(&self, window_id: &str) -> Result<()> {
        let params = serde_json::json!({ "window_id": window_id });
        self.json_rpc_call("focus_window", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw focus_window failed: {}", e))
            })?;
        Ok(())
    }

    async fn get_window_bounds(&self, window_id: &str) -> Result<Rect> {
        let params = serde_json::json!({ "window_id": window_id });
        let result = self
            .json_rpc_call("get_window_bounds", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw get_window_bounds failed: {}", e))
            })?;
        serde_json::from_value(result)
            .map_err(|e| crate::error::action_err(format!("invalid rect: {}", e)))
    }

    async fn resize_window(&self, window_id: &str, width: u32, height: u32) -> Result<()> {
        let params = serde_json::json!({
            "window_id": window_id,
            "width": width,
            "height": height,
        });
        self.json_rpc_call("resize_window", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw resize_window failed: {}", e))
            })?;
        Ok(())
    }

    async fn move_window(&self, window_id: &str, x: i32, y: i32) -> Result<()> {
        let params = serde_json::json!({
            "window_id": window_id,
            "x": x,
            "y": y,
        });
        self.json_rpc_call("move_window", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw move_window failed: {}", e)))?;
        Ok(())
    }

    async fn close_window(&self, window_id: &str) -> Result<()> {
        let params = serde_json::json!({ "window_id": window_id });
        self.json_rpc_call("close_window", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw close_window failed: {}", e))
            })?;
        Ok(())
    }

    async fn minimize_window(&self, window_id: &str) -> Result<()> {
        let params = serde_json::json!({ "window_id": window_id });
        self.json_rpc_call("minimize_window", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw minimize_window failed: {}", e))
            })?;
        Ok(())
    }

    async fn maximize_window(&self, window_id: &str) -> Result<()> {
        let params = serde_json::json!({ "window_id": window_id });
        self.json_rpc_call("maximize_window", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw maximize_window failed: {}", e))
            })?;
        Ok(())
    }

    async fn restore_window(&self, window_id: &str) -> Result<()> {
        let params = serde_json::json!({ "window_id": window_id });
        self.json_rpc_call("restore_window", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw restore_window failed: {}", e))
            })?;
        Ok(())
    }

    async fn find_element(&self, selector: &ElementSelector) -> Result<Vec<ElementInfo>> {
        let params = serde_json::to_value(selector)
            .map_err(|e| crate::error::action_err(format!("serialization error: {}", e)))?;
        let result = self
            .json_rpc_call("find_element", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw find_element failed: {}", e))
            })?;
        serde_json::from_value(result)
            .map_err(|e| crate::error::action_err(format!("invalid element info: {}", e)))
    }

    async fn get_element_text(&self, element_id: &str) -> Result<String> {
        let params = serde_json::json!({ "element_id": element_id });
        let result = self
            .json_rpc_call("get_element_text", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw get_element_text failed: {}", e))
            })?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| crate::error::action_err("invalid text response"))
    }

    async fn click_element(&self, element_id: &str) -> Result<()> {
        let params = serde_json::json!({ "element_id": element_id });
        self.json_rpc_call("click_element", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw click_element failed: {}", e))
            })?;
        Ok(())
    }

    async fn get_element_bounds(&self, element_id: &str) -> Result<Rect> {
        let params = serde_json::json!({ "element_id": element_id });
        let result = self
            .json_rpc_call("get_element_bounds", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw get_element_bounds failed: {}", e))
            })?;
        serde_json::from_value(result)
            .map_err(|e| crate::error::action_err(format!("invalid rect: {}", e)))
    }

    async fn wait_for_element(
        &self,
        selector: &ElementSelector,
        timeout_ms: u64,
    ) -> Result<ElementInfo> {
        let start = std::time::Instant::now();
        loop {
            let elements = self.find_element(selector).await?;
            if let Some(elem) = elements.into_iter().next() {
                return Ok(elem);
            }
            if start.elapsed().as_millis() as u64 >= timeout_ms {
                return Err(crate::error::timeout_err(format!(
                    "element not found within {}ms",
                    timeout_ms
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn ocr_region(&self, image: &[u8], language: Option<&str>) -> Result<String> {
        let image_b64 = base64_encode(image);
        let params = serde_json::json!({
            "image": image_b64,
            "language": language,
        });
        let result = self
            .json_rpc_call("ocr_region", params)
            .await
            .map_err(|e| crate::error::action_err(format!("OpenClaw OCR failed: {}", e)))?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| crate::error::action_err("invalid OCR response"))
    }

    async fn find_text_on_screen(&self, text: &str, region: Option<Rect>) -> Result<Vec<Rect>> {
        let params = serde_json::json!({
            "text": text,
            "region": region,
        });
        let result = self
            .json_rpc_call("find_text_on_screen", params)
            .await
            .map_err(|e| {
                crate::error::action_err(format!("OpenClaw find_text_on_screen failed: {}", e))
            })?;
        serde_json::from_value(result)
            .map_err(|e| crate::error::action_err(format!("invalid rects: {}", e)))
    }

    async fn verify_state(&self, _expected: &StateVerification) -> Result<bool> {
        Err(crate::error::unsupported_err(
            "verification delegated to VerificationEngine",
        ))
    }

    async fn recover(&self, _error: &str) -> Result<bool> {
        Err(crate::error::unsupported_err(
            "recovery delegated to RecoveryEngine",
        ))
    }

    async fn get_backend_capabilities(&self) -> Vec<AutomationCapability> {
        vec![
            AutomationCapability::Mouse,
            AutomationCapability::Keyboard,
            AutomationCapability::ScreenCapture,
            AutomationCapability::WindowManagement,
            AutomationCapability::ElementDetection,
            AutomationCapability::Ocr,
        ]
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let combined = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((combined >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((combined >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((combined >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(combined & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    for chunk in s.as_bytes().chunks(4) {
        if chunk.len() < 4 {
            break;
        }
        let mut vals = [0u32; 4];
        for (i, &b) in chunk.iter().enumerate() {
            if b == b'=' {
                vals[i] = 0;
            } else if let Some(pos) = CHARS.iter().position(|&c| c == b) {
                vals[i] = pos as u32;
            } else {
                return None;
            }
        }
        let combined = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6) | vals[3];
        result.push((combined >> 16) as u8);
        if chunk[2] != b'=' {
            result.push((combined >> 8) as u8);
        }
        if chunk[3] != b'=' {
            result.push(combined as u8);
        }
    }
    Some(result)
}
