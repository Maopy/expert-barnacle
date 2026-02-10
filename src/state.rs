use std::collections::HashSet;
use std::sync::Arc;

use crate::models::{Account, FileEntry};
use crate::webdav::WebDavClient;

/// How to display files in the main panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Grid,
}

/// Clipboard operation type.
#[derive(Debug, Clone)]
pub enum ClipboardOp {
    Copy(Vec<String>),
    Cut(Vec<String>),
}

/// Global application state shared across views.
pub struct AppState {
    // ── Accounts ──────────────────────────────────────────────
    pub accounts: Vec<Account>,
    pub active_account_index: Option<usize>,

    // ── Navigation ────────────────────────────────────────────
    pub current_path: String,
    pub path_history: Vec<String>,
    pub history_index: usize,

    // ── File listing ──────────────────────────────────────────
    pub entries: Vec<FileEntry>,
    pub selected_indices: HashSet<usize>,
    pub loading: bool,
    pub error: Option<String>,

    // ── UI state ──────────────────────────────────────────────
    pub view_mode: ViewMode,
    pub sidebar_collapsed: bool,
    pub preview_visible: bool,
    pub sort_column: String,
    pub sort_ascending: bool,

    // ── Clipboard ─────────────────────────────────────────────
    pub clipboard: Option<ClipboardOp>,

    // ── WebDAV client (set when an account is active) ─────────
    pub webdav_client: Option<Arc<WebDavClient>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            active_account_index: None,
            current_path: "/".to_string(),
            path_history: vec!["/".to_string()],
            history_index: 0,
            entries: Vec::new(),
            selected_indices: HashSet::new(),
            loading: false,
            error: None,
            view_mode: ViewMode::List,
            sidebar_collapsed: false,
            preview_visible: false,
            sort_column: "name".to_string(),
            sort_ascending: true,
            clipboard: None,
            webdav_client: None,
        }
    }

    /// Get the currently active account, if any.
    pub fn active_account(&self) -> Option<&Account> {
        self.active_account_index
            .and_then(|idx| self.accounts.get(idx))
    }

    /// Navigate to a new path, adding to history.
    pub fn navigate_to(&mut self, path: String) {
        // Trim history after current index
        if self.history_index + 1 < self.path_history.len() {
            self.path_history.truncate(self.history_index + 1);
        }
        self.path_history.push(path.clone());
        self.history_index = self.path_history.len() - 1;
        self.current_path = path;
        self.selected_indices.clear();
    }

    /// Navigate back in history.
    pub fn navigate_back(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_path = self.path_history[self.history_index].clone();
            self.selected_indices.clear();
            true
        } else {
            false
        }
    }

    /// Navigate forward in history.
    pub fn navigate_forward(&mut self) -> bool {
        if self.history_index + 1 < self.path_history.len() {
            self.history_index += 1;
            self.current_path = self.path_history[self.history_index].clone();
            self.selected_indices.clear();
            true
        } else {
            false
        }
    }

    /// Navigate to the parent directory.
    pub fn navigate_up(&mut self) -> bool {
        let path = self.current_path.trim_end_matches('/');
        if let Some(pos) = path.rfind('/') {
            let parent = if pos == 0 {
                "/".to_string()
            } else {
                path[..pos].to_string()
            };
            if parent != self.current_path {
                self.navigate_to(parent);
                return true;
            }
        }
        false
    }

    /// Check if we can navigate back.
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    /// Check if we can navigate forward.
    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.path_history.len()
    }

    /// Check if we can navigate up.
    pub fn can_go_up(&self) -> bool {
        self.current_path != "/"
    }

    /// Get the currently selected entries.
    pub fn selected_entries(&self) -> Vec<&FileEntry> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.entries.get(idx))
            .collect()
    }

    /// Sort entries by the current sort column and direction.
    pub fn sort_entries(&mut self) {
        let ascending = self.sort_ascending;
        match self.sort_column.as_str() {
            "name" => self.entries.sort_by(|a, b| {
                // Directories first, then alphabetical
                let dir_cmp = b.is_directory.cmp(&a.is_directory);
                if dir_cmp != std::cmp::Ordering::Equal {
                    return dir_cmp;
                }
                let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if ascending { cmp } else { cmp.reverse() }
            }),
            "size" => self.entries.sort_by(|a, b| {
                let dir_cmp = b.is_directory.cmp(&a.is_directory);
                if dir_cmp != std::cmp::Ordering::Equal {
                    return dir_cmp;
                }
                let cmp = a.size.unwrap_or(0).cmp(&b.size.unwrap_or(0));
                if ascending { cmp } else { cmp.reverse() }
            }),
            "modified" => self.entries.sort_by(|a, b| {
                let dir_cmp = b.is_directory.cmp(&a.is_directory);
                if dir_cmp != std::cmp::Ordering::Equal {
                    return dir_cmp;
                }
                let cmp = a.last_modified.cmp(&b.last_modified);
                if ascending { cmp } else { cmp.reverse() }
            }),
            "type" => self.entries.sort_by(|a, b| {
                let dir_cmp = b.is_directory.cmp(&a.is_directory);
                if dir_cmp != std::cmp::Ordering::Equal {
                    return dir_cmp;
                }
                let ext_a = a.extension().unwrap_or_default();
                let ext_b = b.extension().unwrap_or_default();
                let cmp = ext_a.cmp(&ext_b);
                if ascending { cmp } else { cmp.reverse() }
            }),
            _ => {}
        }
    }
}
