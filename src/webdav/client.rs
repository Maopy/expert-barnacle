use std::io::Read;

use base64::Engine;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use super::types::{DavError, DavResource};
use super::xml::{parse_multistatus, PROPFIND_ALL_PROPS};

/// A synchronous WebDAV client built on top of `ureq`.
///
/// All methods are blocking and should be called from a background thread
/// (e.g. via `smol::unblock`).
#[derive(Clone)]
pub struct WebDavClient {
    agent: ureq::Agent,
    base_url: String,
    auth_header: Option<String>,
}

impl WebDavClient {
    /// Create a new WebDAV client.
    ///
    /// - `base_url`: The root URL of the WebDAV server (e.g. "https://example.com/dav/").
    /// - `username` / `password`: Credentials for Basic auth. Pass empty strings for no auth.
    pub fn new(base_url: &str, username: &str, password: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout_read(std::time::Duration::from_secs(30))
            .timeout_write(std::time::Duration::from_secs(30))
            .build();

        let auth_header = if !username.is_empty() {
            let credentials = format!("{}:{}", username, password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
            Some(format!("Basic {}", encoded))
        } else {
            None
        };

        Self {
            agent,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header,
        }
    }

    /// Test the connection to the WebDAV server.
    pub fn test_connection(&self) -> Result<(), DavError> {
        self.propfind("/", "0")?;
        Ok(())
    }

    /// List directory contents via PROPFIND.
    ///
    /// - `path`: The directory path relative to base_url.
    /// - `depth`: "0" for the resource itself, "1" for children, "infinity" for recursive.
    pub fn propfind(&self, path: &str, depth: &str) -> Result<Vec<DavResource>, DavError> {
        let url = self.build_url(path);
        let mut req = self
            .agent
            .request("PROPFIND", &url)
            .set("Depth", depth)
            .set("Content-Type", "application/xml; charset=utf-8");

        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        let resp = req.send_string(PROPFIND_ALL_PROPS)?;
        let body = resp.into_string()?;
        parse_multistatus(&body)
    }

    /// List the contents of a directory (depth=1), excluding the directory itself.
    pub fn list(&self, path: &str) -> Result<Vec<DavResource>, DavError> {
        let mut resources = self.propfind(path, "1")?;
        // The first entry is usually the directory itself; remove it.
        let normalized_path = normalize_path(path);
        resources.retain(|r| {
            let r_path = normalize_path(&r.href);
            r_path != normalized_path
        });
        Ok(resources)
    }

    /// Download a file and return its bytes.
    pub fn get(&self, path: &str) -> Result<Vec<u8>, DavError> {
        let url = self.build_url(path);
        let mut req = self.agent.get(&url);
        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        let resp = req.call()?;
        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Download a file with progress callback.
    /// The callback receives (bytes_read, total_bytes_option).
    pub fn get_with_progress(
        &self,
        path: &str,
        mut on_progress: impl FnMut(u64, Option<u64>),
    ) -> Result<Vec<u8>, DavError> {
        let url = self.build_url(path);
        let mut req = self.agent.get(&url);
        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        let resp = req.call()?;
        let total = resp
            .header("Content-Length")
            .and_then(|s| s.parse::<u64>().ok());
        let mut reader = resp.into_reader();
        let mut bytes = Vec::new();
        let mut buf = [0u8; 8192];
        let mut read_total: u64 = 0;

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
            read_total += n as u64;
            on_progress(read_total, total);
        }

        Ok(bytes)
    }

    /// Upload (create or overwrite) a file.
    pub fn put(&self, path: &str, data: &[u8], content_type: &str) -> Result<(), DavError> {
        let url = self.build_url(path);
        let mut req = self
            .agent
            .put(&url)
            .set("Content-Type", content_type);

        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        req.send_bytes(data)?;
        Ok(())
    }

    /// Create a new collection (directory).
    pub fn mkcol(&self, path: &str) -> Result<(), DavError> {
        let url = self.build_url(path);
        let mut req = self.agent.request("MKCOL", &url);
        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        req.call()?;
        Ok(())
    }

    /// Delete a resource (file or collection).
    pub fn delete(&self, path: &str) -> Result<(), DavError> {
        let url = self.build_url(path);
        let mut req = self.agent.delete(&url);
        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        req.call()?;
        Ok(())
    }

    /// Move (rename) a resource.
    pub fn move_resource(&self, from: &str, to: &str) -> Result<(), DavError> {
        let from_url = self.build_url(from);
        let to_url = self.build_url(to);
        let mut req = self
            .agent
            .request("MOVE", &from_url)
            .set("Destination", &to_url)
            .set("Overwrite", "F");

        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        req.call()?;
        Ok(())
    }

    /// Copy a resource.
    pub fn copy_resource(&self, from: &str, to: &str) -> Result<(), DavError> {
        let from_url = self.build_url(from);
        let to_url = self.build_url(to);
        let mut req = self
            .agent
            .request("COPY", &from_url)
            .set("Destination", &to_url)
            .set("Overwrite", "F");

        if let Some(ref auth) = self.auth_header {
            req = req.set("Authorization", auth);
        }

        req.call()?;
        Ok(())
    }

    /// Build a full URL from the base URL and a relative path.
    fn build_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            format!("{}/", self.base_url)
        } else {
            // Encode each path segment separately
            let encoded_segments: Vec<String> = path
                .split('/')
                .map(|segment| {
                    utf8_percent_encode(segment, NON_ALPHANUMERIC)
                        .to_string()
                })
                .collect();
            format!("{}/{}", self.base_url, encoded_segments.join("/"))
        }
    }

    /// Get the base URL of this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Normalize a path for comparison: decode percent-encoding, trim trailing slashes.
fn normalize_path(path: &str) -> String {
    let decoded = percent_encoding::percent_decode_str(path)
        .decode_utf8_lossy()
        .to_string();
    decoded.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client = WebDavClient::new("https://example.com/dav", "user", "pass");
        assert_eq!(client.build_url("/"), "https://example.com/dav/");
        assert_eq!(
            client.build_url("/documents/test.txt"),
            "https://example.com/dav/documents/test%2Etxt"
        );
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/dav/files/"), "/dav/files");
        assert_eq!(normalize_path("/dav/files"), "/dav/files");
        assert_eq!(normalize_path("/dav/my%20file.txt"), "/dav/my file.txt");
    }
}
