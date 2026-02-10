use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::models::{Account, AuthType};

/// Manages WebDAV account persistence.
pub struct AccountService {
    config_path: PathBuf,
}

impl AccountService {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wbh");
        Self {
            config_path: config_dir.join("accounts.json"),
        }
    }

    /// Load all saved accounts from disk.
    pub fn load_accounts(&self) -> Result<Vec<Account>> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }
        let data = std::fs::read_to_string(&self.config_path)
            .context("Failed to read accounts file")?;
        let accounts: Vec<Account> =
            serde_json::from_str(&data).context("Failed to parse accounts file")?;
        Ok(accounts)
    }

    /// Save all accounts to disk.
    pub fn save_accounts(&self, accounts: &[Account]) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let data = serde_json::to_string_pretty(accounts)
            .context("Failed to serialize accounts")?;
        std::fs::write(&self.config_path, data).context("Failed to write accounts file")?;
        Ok(())
    }

    /// Add a new account and save.
    pub fn add_account(&self, accounts: &mut Vec<Account>, account: Account) -> Result<()> {
        accounts.push(account);
        self.save_accounts(accounts)
    }

    /// Update an existing account by ID and save.
    pub fn update_account(&self, accounts: &mut Vec<Account>, updated: Account) -> Result<()> {
        if let Some(existing) = accounts.iter_mut().find(|a| a.id == updated.id) {
            *existing = updated;
        }
        self.save_accounts(accounts)
    }

    /// Remove an account by ID and save.
    pub fn remove_account(&self, accounts: &mut Vec<Account>, id: &str) -> Result<()> {
        accounts.retain(|a| a.id != id);
        self.save_accounts(accounts)
    }

    /// Store password securely in the system keychain.
    pub fn store_password(account: &Account, password: &str) -> Result<()> {
        let entry = keyring::Entry::new(&account.keyring_service(), &account.username)
            .context("Failed to create keyring entry")?;
        entry
            .set_password(password)
            .context("Failed to store password in keychain")?;
        Ok(())
    }

    /// Retrieve password from the system keychain.
    pub fn get_password(account: &Account) -> Result<String> {
        let entry = keyring::Entry::new(&account.keyring_service(), &account.username)
            .context("Failed to create keyring entry")?;
        entry
            .get_password()
            .context("Failed to retrieve password from keychain")
    }

    /// Delete password from the system keychain.
    pub fn delete_password(account: &Account) -> Result<()> {
        let entry = keyring::Entry::new(&account.keyring_service(), &account.username)
            .context("Failed to create keyring entry")?;
        entry
            .delete_credential()
            .context("Failed to delete password from keychain")?;
        Ok(())
    }

    /// Create a demo/test account for development purposes.
    pub fn create_demo_account() -> Account {
        Account::new(
            "Demo Server",
            "https://demo.owncloud.com/remote.php/dav/files/demo/",
            "demo",
            AuthType::Basic,
        )
    }
}
