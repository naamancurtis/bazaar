use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::{BazaarError, Result};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct BazaarCookies {
    access: Mutex<Option<String>>,
    refresh: Mutex<Option<String>>,
}

impl BazaarCookies {
    pub(crate) fn new(
        access_cookie: Option<String>,
        refresh_cookie: Option<String>,
    ) -> Result<Self> {
        let cookies = Self::default();
        cookies.set_access_cookie(access_cookie)?;
        cookies.set_refresh_cookie(refresh_cookie)?;
        Ok(cookies)
    }

    pub(crate) fn set_access_cookie(&self, cookie: Option<String>) -> Result<()> {
        *self
            .access
            .lock()
            .map_err(|e| BazaarError::PoisonConcurrencyError(e.to_string()))? = cookie;
        Ok(())
    }

    pub(crate) fn set_refresh_cookie(&self, cookie: Option<String>) -> Result<()> {
        *self
            .refresh
            .lock()
            .map_err(|e| BazaarError::PoisonConcurrencyError(e.to_string()))? = cookie;
        Ok(())
    }

    pub(crate) fn get_access_cookie(&self) -> Result<Option<String>> {
        Ok(self
            .access
            .lock()
            .map_err(|e| BazaarError::PoisonConcurrencyError(e.to_string()))?
            .clone())
    }

    pub(crate) fn get_refresh_cookie(&self) -> Result<Option<String>> {
        Ok(self
            .refresh
            .lock()
            .map_err(|e| BazaarError::PoisonConcurrencyError(e.to_string()))?
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use claim::assert_none;

    #[test]
    fn get_and_set_refresh_works() -> Result<()> {
        let cookies = BazaarCookies::default();
        assert_none!(cookies.get_refresh_cookie()?);
        cookies.set_refresh_cookie(Some("TOKEN".to_string()))?;
        assert_eq!(cookies.get_refresh_cookie()?, Some("TOKEN".to_string()));
        assert_none!(cookies.get_access_cookie()?);

        cookies.set_access_cookie(Some("DOESNT CHANGE".to_string()))?;
        assert_eq!(cookies.get_refresh_cookie()?, Some("TOKEN".to_string()));
        Ok(())
    }

    #[test]
    fn get_and_set_access_works() -> Result<()> {
        let cookies = BazaarCookies::default();
        assert_none!(cookies.get_access_cookie()?);
        cookies.set_access_cookie(Some("TOKEN".to_string()))?;
        assert_eq!(cookies.get_access_cookie()?, Some("TOKEN".to_string()));
        assert_none!(cookies.get_refresh_cookie()?);

        cookies.set_refresh_cookie(Some("DOESNT CHANGE".to_string()))?;
        assert_eq!(cookies.get_access_cookie()?, Some("TOKEN".to_string()));

        Ok(())
    }
}
