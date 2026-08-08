use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{info, warn};

use crate::error::{Result, VisionError};
use crate::provider::CaptureProvider;
use crate::types::{
    CaptureSource, CaptureSourceKind, CapturedFrame, DisplayInfo, PixelFormat, Rect,
};

struct CaptureState {
    last_frames: VecDeque<CapturedFrame>,
    displays: Vec<DisplayInfo>,
    _active_captures: u32,
}

pub struct CaptureEngine {
    provider: Arc<dyn CaptureProvider>,
    state: RwLock<CaptureState>,
    max_cached_frames: usize,
    dpi_aware: bool,
    multi_monitor: bool,
}

impl CaptureEngine {
    pub fn provider(&self) -> &Arc<dyn CaptureProvider> {
        &self.provider
    }

    pub fn new(provider: Arc<dyn CaptureProvider>) -> Self {
        Self {
            provider,
            state: RwLock::new(CaptureState {
                last_frames: VecDeque::new(),
                displays: Vec::new(),
                _active_captures: 0,
            }),
            max_cached_frames: 5,
            dpi_aware: true,
            multi_monitor: true,
        }
    }

    pub fn with_max_cached_frames(mut self, n: usize) -> Self {
        self.max_cached_frames = n;
        self
    }

    pub fn with_dpi_aware(mut self, enabled: bool) -> Self {
        self.dpi_aware = enabled;
        self
    }

    pub fn with_multi_monitor(mut self, enabled: bool) -> Self {
        self.multi_monitor = enabled;
        self
    }

    pub async fn refresh_displays(&self) -> Result<Vec<DisplayInfo>> {
        let displays = self.provider.list_displays().await?;
        {
            let mut state = self.state.write();
            state.displays = displays.clone();
        }
        Ok(displays)
    }

    pub async fn capture_full_screen(&self, display_index: u32) -> Result<CapturedFrame> {
        let frame = self.provider.capture_full_screen(display_index).await?;
        self.cache_frame(&frame);
        Ok(frame)
    }

    pub async fn capture_window(&self, window_id: &str) -> Result<CapturedFrame> {
        if !self.provider.supports_window_capture() {
            return Err(VisionError::UnsupportedOperation(
                "capture provider does not support window capture".into(),
            ));
        }
        let frame = self.provider.capture_window(window_id).await?;
        self.cache_frame(&frame);
        Ok(frame)
    }

    pub async fn capture_region(&self, display_index: u32, region: Rect) -> Result<CapturedFrame> {
        if !self.provider.supports_region_capture() {
            let full = self.provider.capture_full_screen(display_index).await?;
            return crop_frame(&full, region);
        }
        let frame = self.provider.capture_region(display_index, region).await?;
        self.cache_frame(&frame);
        Ok(frame)
    }

    pub async fn capture_all_displays(&self) -> Result<Vec<CapturedFrame>> {
        if !self.multi_monitor {
            let frame = self.capture_full_screen(0).await?;
            return Ok(vec![frame]);
        }
        let displays = self.refresh_displays().await?;
        let mut frames = Vec::with_capacity(displays.len());
        for disp in &displays {
            match self.capture_full_screen(disp.index).await {
                Ok(frame) => frames.push(frame),
                Err(e) => warn!("Failed to capture display {}: {}", disp.index, e),
            }
        }
        if frames.is_empty() {
            return Err(VisionError::CaptureFailed(
                "no displays could be captured".into(),
            ));
        }
        Ok(frames)
    }

    pub fn get_last_frame(&self, display_index: u32) -> Option<CapturedFrame> {
        let state = self.state.read();
        state
            .last_frames
            .iter()
            .find(|f| f.monitor_index == display_index)
            .cloned()
    }

    pub fn get_cached_displays(&self) -> Vec<DisplayInfo> {
        self.state.read().displays.clone()
    }

    pub fn clear_cache(&self) {
        let mut state = self.state.write();
        state.last_frames.clear();
        info!("Capture cache cleared");
    }

    fn cache_frame(&self, frame: &CapturedFrame) {
        let mut state = self.state.write();
        state
            .last_frames
            .retain(|f| f.monitor_index != frame.monitor_index);
        state.last_frames.push_back(frame.clone());
        while state.last_frames.len() > self.max_cached_frames {
            state.last_frames.pop_front();
        }
    }
}

pub fn crop_frame(frame: &CapturedFrame, region: Rect) -> Result<CapturedFrame> {
    let bytes_per_pixel = match frame.format {
        PixelFormat::Rgba | PixelFormat::Bgra => 4,
        PixelFormat::Rgb | PixelFormat::Bgr => 3,
        PixelFormat::Grayscale => 1,
    };

    let rx = region.x().max(0) as u32;
    let ry = region.y().max(0) as u32;
    let rw = region.width().min(frame.width.saturating_sub(rx));
    let rh = region.height().min(frame.height.saturating_sub(ry));

    if rw == 0 || rh == 0 {
        return Err(VisionError::InvalidParameter(
            "crop region is empty or out of bounds".into(),
        ));
    }

    let row_size = frame.stride as usize;
    let crop_row_size = rw as usize * bytes_per_pixel;
    let mut cropped = Vec::with_capacity(rh as usize * crop_row_size);

    for y in ry..ry + rh {
        let src_start = y as usize * row_size + rx as usize * bytes_per_pixel;
        let src_end = src_start + crop_row_size;
        if src_end <= frame.data.len() {
            cropped.extend_from_slice(&frame.data[src_start..src_end]);
        }
    }

    Ok(CapturedFrame {
        id: uuid::Uuid::new_v4().to_string(),
        data: cropped,
        width: rw,
        height: rh,
        stride: crop_row_size as u32,
        format: frame.format,
        source: CaptureSource {
            kind: CaptureSourceKind::Region,
            display_index: Some(frame.monitor_index),
            window_id: None,
            region: Some(region),
        },
        timestamp: chrono::Utc::now(),
        dpi_scale: frame.dpi_scale,
        monitor_index: frame.monitor_index,
    })
}

pub fn compute_frame_diff(
    before: &CapturedFrame,
    after: &CapturedFrame,
    threshold: u8,
) -> Vec<Rect> {
    if before.format != after.format || before.width != after.width || before.height != after.height
    {
        return vec![Rect::new(0, 0, after.width, after.height)];
    }

    let bpp = match before.format {
        PixelFormat::Rgba | PixelFormat::Bgra => 4,
        PixelFormat::Rgb | PixelFormat::Bgr => 3,
        PixelFormat::Grayscale => 1,
    };

    let mut changed_regions = Vec::new();
    let mut in_diff = false;
    let mut diff_start_x = 0u32;
    let mut diff_start_y = 0u32;
    let mut diff_end_x = 0u32;

    let scan_stride = 16.min(before.width);
    let min_changed_area = 4u32;

    for y in (0..before.height).step_by(scan_stride as usize) {
        let mut row_changed = false;
        for x in (0..before.width).step_by(scan_stride as usize) {
            let before_idx = (y as usize * before.stride as usize) + (x as usize * bpp);
            let after_idx = (y as usize * after.stride as usize) + (x as usize * bpp);

            if before_idx + bpp > before.data.len() || after_idx + bpp > after.data.len() {
                continue;
            }

            let changed = (0..bpp.min(3)).any(|c| {
                let diff: u16 = (before.data[before_idx + c] as i16
                    - after.data[after_idx + c] as i16)
                    .unsigned_abs();
                diff > threshold as u16
            });

            if changed {
                if !in_diff {
                    diff_start_x = x;
                    diff_start_y = y;
                    diff_end_x = x + scan_stride;
                    in_diff = true;
                } else {
                    diff_end_x = x + scan_stride;
                }
                row_changed = true;
            }
        }

        if in_diff && !row_changed {
            let w = diff_end_x - diff_start_x;
            let h = scan_stride;
            if w >= min_changed_area && h >= min_changed_area {
                changed_regions.push(Rect::new(
                    diff_start_x as i32,
                    diff_start_y as i32,
                    w,
                    h.max(scan_stride),
                ));
            }
            in_diff = false;
        }
    }

    if in_diff {
        let w = diff_end_x - diff_start_x;
        if w >= min_changed_area {
            changed_regions.push(Rect::new(
                diff_start_x as i32,
                diff_start_y as i32,
                w,
                scan_stride,
            ));
        }
    }

    changed_regions
}

pub fn convert_pixel_format(
    data: &[u8],
    width: u32,
    height: u32,
    from: PixelFormat,
    to: PixelFormat,
) -> Vec<u8> {
    if from == to {
        return data.to_vec();
    }

    let pixel_count = width as usize * height as usize;
    match (from, to) {
        (PixelFormat::Bgra, PixelFormat::Rgba) | (PixelFormat::Rgba, PixelFormat::Bgra) => {
            let mut result = data.to_vec();
            for i in 0..pixel_count {
                let idx = i * 4;
                result.swap(idx, idx + 2);
            }
            result
        }
        (PixelFormat::Bgr, PixelFormat::Rgb) | (PixelFormat::Rgb, PixelFormat::Bgr) => {
            let mut result = data.to_vec();
            for i in 0..pixel_count {
                let idx = i * 3;
                result.swap(idx, idx + 2);
            }
            result
        }
        (PixelFormat::Rgb, PixelFormat::Rgba) => {
            let mut result = Vec::with_capacity(data.len() + pixel_count);
            for i in 0..pixel_count {
                let idx = i * 3;
                result.extend_from_slice(&data[idx..idx + 3]);
                result.push(255);
            }
            result
        }
        (PixelFormat::Bgr, PixelFormat::Bgra) => {
            let mut result = Vec::with_capacity(data.len() + pixel_count);
            for i in 0..pixel_count {
                let idx = i * 3;
                result.extend_from_slice(&data[idx..idx + 3]);
                result.push(255);
            }
            result
        }
        (PixelFormat::Rgba, PixelFormat::Rgb) => {
            let mut result = Vec::with_capacity(pixel_count * 3);
            for i in 0..pixel_count {
                let idx = i * 4;
                result.extend_from_slice(&data[idx..idx + 3]);
            }
            result
        }
        (PixelFormat::Bgra, PixelFormat::Bgr) => {
            let mut result = Vec::with_capacity(pixel_count * 3);
            for i in 0..pixel_count {
                let idx = i * 4;
                result.extend_from_slice(&data[idx..idx + 3]);
            }
            result
        }
        _ => {
            tracing::warn!(
                "Unsupported pixel format conversion: {:?} -> {:?}",
                from,
                to
            );
            data.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CaptureSourceKind;

    struct TestCaptureProvider;

    #[async_trait::async_trait]
    impl CaptureProvider for TestCaptureProvider {
        async fn capture_full_screen(&self, _display_index: u32) -> Result<CapturedFrame> {
            Ok(CapturedFrame {
                id: "test".into(),
                data: vec![128u8; 640 * 480 * 4],
                width: 640,
                height: 480,
                stride: 640 * 4,
                format: PixelFormat::Rgba,
                source: CaptureSource {
                    kind: CaptureSourceKind::FullScreen,
                    display_index: Some(0),
                    window_id: None,
                    region: None,
                },
                timestamp: chrono::Utc::now(),
                dpi_scale: 1.0,
                monitor_index: 0,
            })
        }

        async fn capture_window(&self, _window_id: &str) -> Result<CapturedFrame> {
            Err(VisionError::UnsupportedOperation(
                "no window capture".into(),
            ))
        }

        async fn capture_region(
            &self,
            _display_index: u32,
            _region: Rect,
        ) -> Result<CapturedFrame> {
            Err(VisionError::UnsupportedOperation(
                "no region capture".into(),
            ))
        }

        async fn list_displays(&self) -> Result<Vec<DisplayInfo>> {
            Ok(vec![DisplayInfo {
                index: 0,
                bounds: Rect::new(0, 0, 1920, 1080),
                dpi_scale: 1.0,
                is_primary: true,
                name: "Test Display".into(),
                is_attached: true,
            }])
        }

        async fn screen_size(&self, _display_index: u32) -> Result<(u32, u32)> {
            Ok((1920, 1080))
        }

        fn supported_pixel_formats(&self) -> Vec<PixelFormat> {
            vec![PixelFormat::Rgba]
        }

        fn name(&self) -> &str {
            "test-capture"
        }

        fn supports_window_capture(&self) -> bool {
            false
        }

        fn supports_region_capture(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn test_capture_engine_creates() {
        let provider = std::sync::Arc::new(TestCaptureProvider);
        let engine = CaptureEngine::new(provider);
        assert_eq!(engine.get_cached_displays().len(), 0);
    }

    #[tokio::test]
    async fn test_capture_full_screen() {
        let provider = std::sync::Arc::new(TestCaptureProvider);
        let engine = CaptureEngine::new(provider);
        let frame = engine.capture_full_screen(0).await.unwrap();
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
    }

    #[tokio::test]
    async fn test_capture_window_unsupported() {
        let provider = std::sync::Arc::new(TestCaptureProvider);
        let engine = CaptureEngine::new(provider);
        let result = engine.capture_window("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_displays() {
        let provider = std::sync::Arc::new(TestCaptureProvider);
        let engine = CaptureEngine::new(provider);
        let displays = engine.refresh_displays().await.unwrap();
        assert_eq!(displays.len(), 1);
        assert_eq!(displays[0].index, 0);
    }

    #[test]
    fn test_pixel_format_convert_bgra_to_rgba() {
        let data = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let result = convert_pixel_format(&data, 2, 1, PixelFormat::Bgra, PixelFormat::Rgba);
        assert_eq!(result, vec![0, 0, 255, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn test_pixel_format_identity() {
        let data = vec![1, 2, 3, 4];
        let result = convert_pixel_format(&data, 1, 1, PixelFormat::Rgba, PixelFormat::Rgba);
        assert_eq!(result, data);
    }

    #[test]
    fn test_compute_frame_diff_identical() {
        let frame = CapturedFrame {
            id: "a".into(),
            data: vec![128u8; 100],
            width: 10,
            height: 10,
            stride: 10,
            format: PixelFormat::Grayscale,
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(0),
                window_id: None,
                region: None,
            },
            timestamp: chrono::Utc::now(),
            dpi_scale: 1.0,
            monitor_index: 0,
        };
        let diffs = compute_frame_diff(&frame, &frame, 10);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_compute_frame_diff_different() {
        let before = CapturedFrame {
            id: "a".into(),
            data: vec![0u8; 160 * 120],
            width: 160,
            height: 120,
            stride: 160,
            format: PixelFormat::Grayscale,
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(0),
                window_id: None,
                region: None,
            },
            timestamp: chrono::Utc::now(),
            dpi_scale: 1.0,
            monitor_index: 0,
        };
        let mut after_data = vec![0u8; 160 * 120];
        // Change a pixel at a position that falls on the scan grid (x=0, y=0)
        after_data[0] = 255;
        let after = CapturedFrame {
            id: "b".into(),
            data: after_data,
            width: 160,
            height: 120,
            stride: 160,
            format: PixelFormat::Grayscale,
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(0),
                window_id: None,
                region: None,
            },
            timestamp: chrono::Utc::now(),
            dpi_scale: 1.0,
            monitor_index: 0,
        };
        let diffs = compute_frame_diff(&before, &after, 10);
        assert!(!diffs.is_empty());
    }
}
