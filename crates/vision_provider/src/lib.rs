pub mod windows_capture;
pub mod windows_ocr;

pub use voxy_provider_core as core_traits;
pub use windows_capture::WindowsGraphicsCaptureProvider;
pub use windows_ocr::WindowsOcrProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reexports() {
        let _cap = WindowsGraphicsCaptureProvider::new();
        let _ocr = WindowsOcrProvider::new();
    }
}
