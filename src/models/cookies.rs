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
            .access
            .lock()
            .map_err(|e| BazaarError::PoisonConcurrencyError(e.to_string()))?
            .clone())
    }

    /// The way the application is currently coded, if either the access
    /// cookie or the refresh cookie are `Some` then it will automatically
    /// update the cookie on the response.
    ///
    /// For most query/mutations, (other than auth related ones) we don't
    /// want to modify the cookies in any way, so this state needs to be set to
    /// `None` once the cookie has been verified and the customer should
    /// have access to that resource
    pub(crate) fn set_cookies_to_not_be_changed(&self) -> Result<()> {
        self.set_refresh_cookie(None)?;
        self.set_access_cookie(None)?;
        Ok(())
    }
}
