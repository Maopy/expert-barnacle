use chrono::{DateTime, Utc};

use crate::webdav::DavResource;

/// A file or directory entry displayed in the UI.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// The display name.
    pub name: String,
    /// Full path on the WebDAV server.
    pub path: String,
    /// Whether this is a directory (collection).
    pub is_directory: bool,
    /// File size in bytes (None for directories).
    pub size: Option<u64>,
    /// MIME content type.
    pub content_type: Option<String>,
    /// Last modification timestamp.
    pub last_modified: Option<DateTime<Utc>>,
    /// ETag for change detection.
    pub etag: Option<String>,
}

impl FileEntry {
    /// Convert from a DavResource.
    pub fn from_dav_resource(resource: &DavResource) -> Self {
        Self {
            name: resource.name(),
            path: resource.href.clone(),
            is_directory: resource.is_collection,
            size: resource.content_length,
            content_type: resource.content_type.clone(),
            last_modified: resource.last_modified,
            etag: resource.etag.clone(),
        }
    }

    /// Get a display-friendly file extension.
    pub fn extension(&self) -> Option<String> {
        if self.is_directory {
            None
        } else {
            self.name
                .rsplit('.')
                .next()
                .filter(|ext| ext.len() < 10)
                .map(|ext| ext.to_lowercase())
        }
    }
}
