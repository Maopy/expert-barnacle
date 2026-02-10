use chrono::{DateTime, Utc};
use std::fmt;

/// Represents a resource (file or collection) on a WebDAV server.
#[derive(Debug, Clone)]
pub struct DavResource {
    /// The href (path) of the resource on the server.
    pub href: String,
    /// Display name of the resource.
    pub display_name: Option<String>,
    /// Whether this resource is a collection (directory).
    pub is_collection: bool,
    /// Content length in bytes (None for collections).
    pub content_length: Option<u64>,
    /// MIME content type.
    pub content_type: Option<String>,
    /// Last modification timestamp.
    pub last_modified: Option<DateTime<Utc>>,
    /// ETag for change detection.
    pub etag: Option<String>,
}

impl DavResource {
    /// Extract the file/directory name from the href.
    pub fn name(&self) -> String {
        if let Some(ref name) = self.display_name {
            if !name.is_empty() {
                return name.clone();
            }
        }
        // Fallback: extract from href
        let path = self.href.trim_end_matches('/');
        path.rsplit('/')
            .next()
            .map(|s| {
                percent_encoding::percent_decode_str(s)
                    .decode_utf8_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "/".to_string())
    }
}

/// Errors that can occur during WebDAV operations.
#[derive(Debug)]
pub enum DavError {
    /// HTTP-level error with status code.
    Http { status: u16, message: String },
    /// Failed to parse XML response.
    XmlParse(String),
    /// Network / transport error.
    Network(String),
    /// Authentication failed (401/403).
    AuthFailed,
    /// Resource not found (404).
    NotFound(String),
    /// Conflict (409) — e.g. creating a collection whose parent doesn't exist.
    Conflict(String),
    /// Server error (5xx).
    ServerError(String),
}

impl fmt::Display for DavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DavError::Http { status, message } => write!(f, "HTTP {}: {}", status, message),
            DavError::XmlParse(msg) => write!(f, "XML parse error: {}", msg),
            DavError::Network(msg) => write!(f, "Network error: {}", msg),
            DavError::AuthFailed => write!(f, "Authentication failed"),
            DavError::NotFound(path) => write!(f, "Not found: {}", path),
            DavError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            DavError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for DavError {}

impl From<ureq::Error> for DavError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Status(401, _) | ureq::Error::Status(403, _) => DavError::AuthFailed,
            ureq::Error::Status(404, resp) => {
                DavError::NotFound(resp.into_string().unwrap_or_default())
            }
            ureq::Error::Status(409, resp) => {
                DavError::Conflict(resp.into_string().unwrap_or_default())
            }
            ureq::Error::Status(status, resp) if status >= 500 => {
                DavError::ServerError(resp.into_string().unwrap_or_default())
            }
            ureq::Error::Status(status, resp) => DavError::Http {
                status,
                message: resp.into_string().unwrap_or_default(),
            },
            ureq::Error::Transport(t) => DavError::Network(t.to_string()),
        }
    }
}

impl From<std::io::Error> for DavError {
    fn from(err: std::io::Error) -> Self {
        DavError::Network(err.to_string())
    }
}
