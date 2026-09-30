use keyring::Entry;
use thiserror::Error;

const SERVICE_NAME: &str = "ferris-agent";

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credential is not configured")]
    NotFound,
    #[error("credential store operation failed")]
    Store,
}

pub trait CredentialBackend: Send + Sync {
    fn get(&self, account: &str) -> Result<String, CredentialError>;
    fn set(&self, account: &str, value: &str) -> Result<(), CredentialError>;
    fn delete(&self, account: &str) -> Result<(), CredentialError>;
}

struct KeyringBackend;

impl KeyringBackend {
    fn entry(account: &str) -> Result<Entry, CredentialError> {
        Entry::new(SERVICE_NAME, account).map_err(|_| CredentialError::Store)
    }
}

impl CredentialBackend for KeyringBackend {
    fn get(&self, account: &str) -> Result<String, CredentialError> {
        match Self::entry(account)?.get_password() {
            Ok(password) => Ok(password),
            Err(keyring::Error::NoEntry) => Err(CredentialError::NotFound),
            Err(_) => Err(CredentialError::Store),
        }
    }

    fn set(&self, account: &str, value: &str) -> Result<(), CredentialError> {
        Self::entry(account)?
            .set_password(value)
            .map_err(|_| CredentialError::Store)
    }

    fn delete(&self, account: &str) -> Result<(), CredentialError> {
        match Self::entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(CredentialError::Store),
        }
    }
}

pub struct CredentialStore {
    backend: Box<dyn CredentialBackend>,
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore {
    pub fn new() -> Self {
        Self::with_backend(KeyringBackend)
    }

    pub fn with_backend(backend: impl CredentialBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }

    pub fn get_api_key(&self, provider: &str) -> Result<String, CredentialError> {
        self.backend.get(provider)
    }

    pub fn set_api_key(&self, provider: &str, api_key: &str) -> Result<(), CredentialError> {
        self.backend.set(provider, api_key)
    }

    pub fn delete_api_key(&self, provider: &str) -> Result<(), CredentialError> {
        self.backend.delete(provider)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct MemoryBackend(Mutex<HashMap<String, String>>);

    impl CredentialBackend for MemoryBackend {
        fn get(&self, account: &str) -> Result<String, CredentialError> {
            self.0
                .lock()
                .unwrap()
                .get(account)
                .cloned()
                .ok_or(CredentialError::NotFound)
        }

        fn set(&self, account: &str, value: &str) -> Result<(), CredentialError> {
            self.0.lock().unwrap().insert(account.into(), value.into());
            Ok(())
        }

        fn delete(&self, account: &str) -> Result<(), CredentialError> {
            self.0.lock().unwrap().remove(account);
            Ok(())
        }
    }

    #[test]
    fn stores_retrieves_and_deletes_credentials() {
        let store = CredentialStore::with_backend(MemoryBackend::default());
        store.set_api_key("deepseek", "test-secret").unwrap();
        assert_eq!(store.get_api_key("deepseek").unwrap(), "test-secret");
        store.delete_api_key("deepseek").unwrap();
        assert!(matches!(
            store.get_api_key("deepseek"),
            Err(CredentialError::NotFound)
        ));
    }
}
