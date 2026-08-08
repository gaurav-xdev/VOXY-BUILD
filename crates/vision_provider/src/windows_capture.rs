use async_trait::async_trait;
use tracing::info;

use voxy_vision::error::{Result, VisionError};
use voxy_vision::provider::CaptureProvider;
use voxy_vision::types::{
    CaptureSource, CaptureSourceKind, CapturedFrame, DisplayInfo, PixelFormat, Rect,
};

pub struct WindowsGraphicsCaptureProvider {
    name: String,
}

impl WindowsGraphicsCaptureProvider {
    pub fn new() -> Self {
        Self {
            name: "windows-graphics-capture".into(),
        }
    }

    unsafe fn capture_dc_inner(display_index: u32) -> Result<CapturedFrame> {
        let hdc_window = unsafe { GetDC(std::ptr::null_mut()) };
        if hdc_window == 0 {
            return Err(VisionError::CaptureFailed("GetDC failed".into()));
        }

        let width = unsafe { GetSystemMetrics(SM_CXSCREEN) as u32 };
        let height = unsafe { GetSystemMetrics(SM_CYSCREEN) as u32 };

        let hdc_mem = unsafe { CreateCompatibleDC(hdc_window) };
        if hdc_mem == 0 {
            unsafe { ReleaseDC(std::ptr::null_mut(), hdc_window) };
            return Err(VisionError::CaptureFailed(
                "CreateCompatibleDC failed".into(),
            ));
        }

        let h_bitmap = unsafe { CreateCompatibleBitmap(hdc_window, width as i32, height as i32) };
        if h_bitmap == 0 {
            unsafe { DeleteDC(hdc_mem) };
            unsafe { ReleaseDC(std::ptr::null_mut(), hdc_window) };
            return Err(VisionError::CaptureFailed(
                "CreateCompatibleBitmap failed".into(),
            ));
        }

        unsafe { SelectObject(hdc_mem, h_bitmap) };
        unsafe {
            BitBlt(
                hdc_mem,
                0,
                0,
                width as i32,
                height as i32,
                hdc_window,
                0,
                0,
                SRCCOPY,
            );
        }

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };

        let row_size = width as usize * 4;
        let data_size = row_size * height as usize;
        let mut data = vec![0u8; data_size];

        let result = unsafe {
            GetDIBits(
                hdc_mem,
                h_bitmap,
                0,
                height,
                data.as_mut_ptr() as *mut std::ffi::c_void,
                &mut bmp_info,
                0,
            )
        };

        unsafe { DeleteObject(h_bitmap) };
        unsafe { DeleteDC(hdc_mem) };
        unsafe { ReleaseDC(std::ptr::null_mut(), hdc_window) };

        if result == 0 {
            return Err(VisionError::CaptureFailed("GetDIBits failed".into()));
        }

        Ok(CapturedFrame {
            id: uuid::Uuid::new_v4().to_string(),
            data,
            width,
            height,
            stride: row_size as u32,
            format: PixelFormat::Bgra,
            source: CaptureSource {
                kind: CaptureSourceKind::FullScreen,
                display_index: Some(display_index),
                window_id: None,
                region: None,
            },
            timestamp: chrono::Utc::now(),
            dpi_scale: 1.0,
            monitor_index: display_index,
        })
    }

    unsafe fn window_capture_inner(window_id: &str) -> Result<CapturedFrame> {
        let hwnd = window_id.parse::<isize>().unwrap_or(0) as *mut std::ffi::c_void;
        if hwnd.is_null() {
            return Err(VisionError::CaptureFailed("invalid window handle".into()));
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let result = unsafe { GetWindowRect(hwnd, &mut rect) };
        if result == 0 {
            return Err(VisionError::CaptureFailed("GetWindowRect failed".into()));
        }

        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        let hdc_window = unsafe { GetDC(hwnd) };
        if hdc_window == 0 {
            return Err(VisionError::CaptureFailed("GetDC for window failed".into()));
        }

        let hdc_mem = unsafe { CreateCompatibleDC(hdc_window) };
        let h_bitmap = unsafe { CreateCompatibleBitmap(hdc_window, width as i32, height as i32) };

        if h_bitmap == 0 {
            unsafe { DeleteDC(hdc_mem) };
            unsafe { ReleaseDC(hwnd, hdc_window) };
            return Err(VisionError::CaptureFailed(
                "CreateCompatibleBitmap failed".into(),
            ));
        }

        unsafe { SelectObject(hdc_mem, h_bitmap) };
        unsafe {
            BitBlt(
                hdc_mem,
                0,
                0,
                width as i32,
                height as i32,
                hdc_window,
                0,
                0,
                SRCCOPY,
            );
        }

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };

        let row_size = width as usize * 4;
        let data_size = row_size * height as usize;
        let mut data = vec![0u8; data_size];

        unsafe {
            GetDIBits(
                hdc_mem,
                h_bitmap,
                0,
                height,
                data.as_mut_ptr() as *mut std::ffi::c_void,
                &mut bmp_info,
                0,
            );
        }

        unsafe { DeleteObject(h_bitmap) };
        unsafe { DeleteDC(hdc_mem) };
        unsafe { ReleaseDC(hwnd, hdc_window) };

        Ok(CapturedFrame {
            id: uuid::Uuid::new_v4().to_string(),
            data,
            width,
            height,
            stride: row_size as u32,
            format: PixelFormat::Bgra,
            source: CaptureSource {
                kind: CaptureSourceKind::Window,
                display_index: None,
                window_id: Some(window_id.to_string()),
                region: None,
            },
            timestamp: chrono::Utc::now(),
            dpi_scale: 1.0,
            monitor_index: 0,
        })
    }
}

impl Default for WindowsGraphicsCaptureProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureProvider for WindowsGraphicsCaptureProvider {
    async fn capture_full_screen(&self, display_index: u32) -> Result<CapturedFrame> {
        info!(
            "Capturing full screen via Win32 GDI (display {})",
            display_index
        );
        if cfg!(windows) {
            Ok(unsafe { Self::capture_dc_inner(display_index)? })
        } else {
            Err(VisionError::UnsupportedOperation(
                "Windows Graphics Capture is only available on Windows".into(),
            ))
        }
    }

    async fn capture_window(&self, window_id: &str) -> Result<CapturedFrame> {
        info!("Capturing window via Win32 GDI: {}", window_id);
        if cfg!(windows) {
            Ok(unsafe { Self::window_capture_inner(window_id)? })
        } else {
            Err(VisionError::UnsupportedOperation(
                "Windows Graphics Capture is only available on Windows".into(),
            ))
        }
    }

    async fn capture_region(&self, display_index: u32, region: Rect) -> Result<CapturedFrame> {
        info!("Capturing region via Win32 GDI: {:?}", region);
        let full = self.capture_full_screen(display_index).await?;
        voxy_vision::capture::crop_frame(&full, region)
    }

    async fn list_displays(&self) -> Result<Vec<DisplayInfo>> {
        if !cfg!(windows) {
            return Ok(vec![]);
        }
        let count = unsafe { GetSystemMetrics(SM_CMONITORS) as u32 };
        let primary_w = unsafe { GetSystemMetrics(SM_CXSCREEN) as u32 };
        let primary_h = unsafe { GetSystemMetrics(SM_CYSCREEN) as u32 };

        let displays = (0..count.max(1))
            .map(|i| DisplayInfo {
                index: i,
                bounds: Rect::new(0, 0, primary_w, primary_h),
                dpi_scale: 1.0,
                is_primary: i == 0,
                name: format!("Display {}", i + 1),
                is_attached: true,
            })
            .collect();

        Ok(displays)
    }

    async fn screen_size(&self, _display_index: u32) -> Result<(u32, u32)> {
        if !cfg!(windows) {
            return Ok((0, 0));
        }
        let w = unsafe { GetSystemMetrics(SM_CXSCREEN) } as u32;
        let h = unsafe { GetSystemMetrics(SM_CYSCREEN) } as u32;
        Ok((w, h))
    }

    fn supported_pixel_formats(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::Bgra]
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn supports_window_capture(&self) -> bool {
        true
    }

    fn supports_region_capture(&self) -> bool {
        true
    }
}

#[allow(non_snake_case, clippy::upper_case_acronyms)]
#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[allow(non_snake_case, clippy::upper_case_acronyms)]
#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

#[allow(non_snake_case, clippy::upper_case_acronyms)]
#[repr(C)]
struct RGBQUAD {
    rgbBlue: u8,
    rgbGreen: u8,
    rgbRed: u8,
    rgbReserved: u8,
}

#[allow(non_snake_case, clippy::upper_case_acronyms)]
#[repr(C)]
struct BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER,
    bmiColors: [RGBQUAD; 1],
}

const SRCCOPY: u32 = 0x00CC0020;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const SM_CMONITORS: i32 = 80;

#[link(name = "user32")]
#[link(name = "gdi32")]
extern "system" {
    fn GetDC(hWnd: *mut std::ffi::c_void) -> isize;
    fn ReleaseDC(hWnd: *mut std::ffi::c_void, hDC: isize) -> i32;
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn CreateCompatibleBitmap(hdc: isize, nWidth: i32, nHeight: i32) -> isize;
    fn SelectObject(hdc: isize, hObject: isize) -> isize;
    fn BitBlt(
        hdcDest: isize,
        xDest: i32,
        yDest: i32,
        wDest: i32,
        hDest: i32,
        hdcSrc: isize,
        xSrc: i32,
        ySrc: i32,
        dwRop: u32,
    ) -> i32;
    fn DeleteDC(hDC: isize) -> i32;
    fn DeleteObject(hObject: isize) -> i32;
    fn GetDIBits(
        hdc: isize,
        hbm: isize,
        start: u32,
        cLines: u32,
        lpvBits: *mut std::ffi::c_void,
        lpbmi: *mut BITMAPINFO,
        usage: u32,
    ) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn GetWindowRect(hWnd: *mut std::ffi::c_void, lpRect: *mut RECT) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creates() {
        let provider = WindowsGraphicsCaptureProvider::new();
        assert_eq!(provider.name(), "windows-graphics-capture");
        assert!(provider.supports_window_capture());
    }

    #[tokio::test]
    async fn test_list_displays() {
        let provider = WindowsGraphicsCaptureProvider::new();
        let displays = provider.list_displays().await.unwrap();
        if cfg!(windows) {
            assert!(!displays.is_empty());
        } else {
            assert!(displays.is_empty());
        }
    }

    #[tokio::test]
    async fn test_screen_size() {
        let provider = WindowsGraphicsCaptureProvider::new();
        let (w, h) = provider.screen_size(0).await.unwrap();
        if cfg!(windows) {
            assert!(w > 0);
            assert!(h > 0);
        } else {
            assert_eq!(w, 0);
            assert_eq!(h, 0);
        }
    }
}
