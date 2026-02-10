use std::sync::Arc;

use anyhow::{Context, Result};

use crate::models::FileEntry;
use crate::webdav::WebDavClient;

/// Provides high-level file operations built on top of the WebDAV client.
pub struct FileOpsService;

impl FileOpsService {
    /// List the contents of a directory and return as FileEntry list.
    pub fn list_directory(
        client: &WebDavClient,
        path: &str,
    ) -> Result<Vec<FileEntry>> {
        let resources = client
            .list(path)
            .context("Failed to list directory")?;
        let entries: Vec<FileEntry> = resources
            .iter()
            .map(FileEntry::from_dav_resource)
            .collect();
        Ok(entries)
    }

    /// Create a new directory.
    pub fn create_directory(client: &WebDavClient, path: &str) -> Result<()> {
        client
            .mkcol(path)
            .context("Failed to create directory")?;
        Ok(())
    }

    /// Delete a file or directory.
    pub fn delete_resource(client: &WebDavClient, path: &str) -> Result<()> {
        client.delete(path).context("Failed to delete resource")?;
        Ok(())
    }

    /// Rename / move a resource.
    pub fn rename_resource(client: &WebDavClient, from: &str, to: &str) -> Result<()> {
        client
            .move_resource(from, to)
            .context("Failed to rename resource")?;
        Ok(())
    }

    /// Copy a resource.
    pub fn copy_resource(client: &WebDavClient, from: &str, to: &str) -> Result<()> {
        client
            .copy_resource(from, to)
            .context("Failed to copy resource")?;
        Ok(())
    }

    /// Download a file to bytes.
    pub fn download_file(client: &WebDavClient, path: &str) -> Result<Vec<u8>> {
        client.get(path).context("Failed to download file")
    }

    /// Download a file with progress callback.
    pub fn download_file_with_progress(
        client: &WebDavClient,
        path: &str,
        on_progress: impl FnMut(u64, Option<u64>),
    ) -> Result<Vec<u8>> {
        client
            .get_with_progress(path, on_progress)
            .context("Failed to download file")
    }

    /// Upload a file from bytes.
    pub fn upload_file(
        client: &WebDavClient,
        path: &str,
        data: &[u8],
        content_type: Option<&str>,
    ) -> Result<()> {
        let ct = content_type.unwrap_or_else(|| {
            mime_guess::from_path(path)
                .first_raw()
                .unwrap_or("application/octet-stream")
        });
        client
            .put(path, data, ct)
            .context("Failed to upload file")?;
        Ok(())
    }

    /// Upload a file from the local filesystem.
    pub fn upload_local_file(
        client: &WebDavClient,
        local_path: &std::path::Path,
        remote_path: &str,
    ) -> Result<()> {
        let data = std::fs::read(local_path).context("Failed to read local file")?;
        let ct = mime_guess::from_path(local_path)
            .first_raw()
            .unwrap_or("application/octet-stream");
        client
            .put(remote_path, &data, ct)
            .context("Failed to upload file")?;
        Ok(())
    }

    /// Download a file to the local filesystem.
    pub fn download_to_local(
        client: &WebDavClient,
        remote_path: &str,
        local_path: &std::path::Path,
    ) -> Result<()> {
        let data = client.get(remote_path).context("Failed to download file")?;
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create local directory")?;
        }
        std::fs::write(local_path, data).context("Failed to write local file")?;
        Ok(())
    }
}
