//! Download manager with progress tracking and concurrency control.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::{Result, RuntimeError};
use parking_lot::RwLock;
use tokio::sync::Semaphore;
use tracing::{error, info};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub id: u64,
    pub url: String,
    pub filename: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub status: DownloadStatus,
}

struct DownloadState {
    url: String,
    filename: String,
    #[allow(dead_code)]
    dest_path: PathBuf,
    bytes_downloaded: Arc<AtomicU64>,
    total_bytes: Option<u64>,
    status: DownloadStatus,
}

pub struct DownloadManager {
    downloads: Arc<RwLock<HashMap<u64, DownloadState>>>,
    counter: AtomicU64,
    semaphore: Arc<Semaphore>,
    download_dir: PathBuf,
}

impl DownloadManager {
    pub fn new(max_concurrent: usize, download_dir: Option<&str>) -> Result<Self> {
        let dir = download_dir.map(PathBuf::from).unwrap_or_else(|| {
            dirs::download_dir().unwrap_or_else(|| std::env::temp_dir().join("voxy_downloads"))
        });
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            downloads: Arc::new(RwLock::new(HashMap::new())),
            counter: AtomicU64::new(0),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            download_dir: dir,
        })
    }

    pub async fn download(&self, url: &str, filename: &str) -> Result<u64> {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let dest = self.download_dir.join(filename);
        let bytes = Arc::new(AtomicU64::new(0));
        self.downloads.write().insert(
            id,
            DownloadState {
                url: url.to_string(),
                filename: filename.to_string(),
                dest_path: dest.clone(),
                bytes_downloaded: bytes.clone(),
                total_bytes: None,
                status: DownloadStatus::Pending,
            },
        );

        let url_clone = url.to_string();
        let filename_clone = filename.to_string();
        let sem = self.semaphore.clone();
        let dls = self.downloads.clone();

        tokio::spawn(async move {
            let _permit = sem.acquire().await;
            if let Some(s) = dls.write().get_mut(&id) {
                s.status = DownloadStatus::Downloading;
            }
            match do_download(&url_clone, &dest, bytes.clone()).await {
                Ok(total) => {
                    if let Some(s) = dls.write().get_mut(&id) {
                        s.status = DownloadStatus::Completed;
                        s.total_bytes = Some(total);
                    }
                    info!("Download completed: {} ({} bytes)", filename_clone, total);
                }
                Err(e) => {
                    if let Some(s) = dls.write().get_mut(&id) {
                        s.status = DownloadStatus::Failed(e.to_string());
                    }
                    error!("Download failed: {} - {}", filename_clone, e);
                }
            }
        });

        info!("Download started: {} -> {} (id={})", url, filename, id);
        Ok(id)
    }

    pub fn cancel(&self, id: u64) -> Result<()> {
        if let Some(s) = self.downloads.write().get_mut(&id) {
            s.status = DownloadStatus::Cancelled;
        }
        Ok(())
    }

    pub fn progress(&self, id: u64) -> Option<DownloadProgress> {
        self.downloads.read().get(&id).map(|s| DownloadProgress {
            id,
            url: s.url.clone(),
            filename: s.filename.clone(),
            bytes_downloaded: s.bytes_downloaded.load(Ordering::Relaxed),
            total_bytes: s.total_bytes,
            status: s.status.clone(),
        })
    }

    pub fn all_downloads(&self) -> Vec<DownloadProgress> {
        self.downloads
            .read()
            .iter()
            .map(|(id, s)| DownloadProgress {
                id: *id,
                url: s.url.clone(),
                filename: s.filename.clone(),
                bytes_downloaded: s.bytes_downloaded.load(Ordering::Relaxed),
                total_bytes: s.total_bytes,
                status: s.status.clone(),
            })
            .collect()
    }

    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }
}

async fn do_download(url: &str, dest: &Path, bytes: Arc<AtomicU64>) -> Result<u64> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| RuntimeError::Download(format!("Client build failed: {}", e)))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| RuntimeError::Download(format!("Request failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(RuntimeError::Download(format!("HTTP {}", resp.status())));
    }
    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| RuntimeError::Download(format!("File create failed: {}", e)))?;
    let mut stream = resp.bytes_stream();
    let mut downloaded = 0u64;
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| RuntimeError::Download(format!("Stream error: {}", e)))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| RuntimeError::Download(format!("Write error: {}", e)))?;
        downloaded += chunk.len() as u64;
        bytes.store(downloaded, Ordering::Relaxed);
    }
    file.flush()
        .await
        .map_err(|e| RuntimeError::Download(format!("Flush error: {}", e)))?;
    Ok(downloaded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_manager_creation() {
        assert!(DownloadManager::new(3, None).is_ok());
    }

    #[test]
    fn download_progress_empty() {
        let mgr = DownloadManager::new(3, None).unwrap();
        assert!(mgr.progress(0).is_none());
        assert!(mgr.all_downloads().is_empty());
    }

    #[test]
    fn download_cancel_nonexistent() {
        let mgr = DownloadManager::new(3, None).unwrap();
        assert!(mgr.cancel(999).is_ok());
    }
}
