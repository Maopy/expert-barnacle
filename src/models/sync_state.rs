use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The synchronization status of a file or directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Fully synchronized.
    Synced,
    /// Local changes waiting to be uploaded.
    PendingUpload,
    /// Remote changes waiting to be downloaded.
    PendingDownload,
    /// Conflicting changes on both sides.
    Conflict,
    /// An error occurred during sync.
    Error(String),
}

/// Synchronization state for a single resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// The account this sync state belongs to.
    pub account_id: String,
    /// Path on the WebDAV server.
    pub remote_path: String,
    /// Path on local filesystem.
    pub local_path: String,
    /// The last known remote ETag.
    pub remote_etag: Option<String>,
    /// The ETag at time of last sync.
    pub synced_etag: Option<String>,
    /// Current sync status.
    pub status: SyncStatus,
    /// When this resource was last successfully synced.
    pub last_synced: Option<DateTime<Utc>>,
}

/// Configuration for a sync folder pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFolder {
    /// The account to sync with.
    pub account_id: String,
    /// Remote path on WebDAV server.
    pub remote_path: String,
    /// Local directory path.
    pub local_path: String,
    /// Whether sync is enabled.
    pub enabled: bool,
    /// Sync interval in seconds (0 = manual only).
    pub interval_secs: u64,
}
