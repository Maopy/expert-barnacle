use serde::{Deserialize, Serialize};

/// Authentication method for a WebDAV account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    Basic,
    Bearer,
    None,
}

impl Default for AuthType {
    fn default() -> Self {
        AuthType::Basic
    }
}

/// A WebDAV server account configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// WebDAV server URL (e.g. "https://example.com/remote.php/dav/files/user/").
    pub url: String,
    /// Username for authentication.
    pub username: String,
    /// Authentication type.
    pub auth_type: AuthType,
}

impl Account {
    /// Create a new account with a generated UUID.
    pub fn new(name: &str, url: &str, username: &str, auth_type: AuthType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            url: url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            auth_type,
        }
    }

    /// The keyring service name for storing this account's password.
    pub fn keyring_service(&self) -> String {
        format!("wbh-webdav-{}", self.id)
    }
}
