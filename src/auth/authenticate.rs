use anyhow::Result;
use argon2::{self, Config, ThreadMode, Variant, Version};
use lazy_static::lazy_static;
use std::env::var;
use tracing::error;

// @TODO - check these are actually okay being `lazy_static` - if the server
// is left up and running for a long time, but we wanted to cycle keys every x
// days, would this pick up on the changes? or would it store a constant value
// for the whole period of time the server is up
lazy_static! {
    pub static ref SECRET_KEY: String = {
        let secret_key: Result<String, ()> = var("SECRET_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve secret key");
            panic!("no SECRET KEY was provided");
        });
        secret_key.unwrap()
    };
    pub static ref SALT: String = {
        let salt: Result<String, ()> = var("SALT").map_err(|e| {
            error!(err = ?e, "failed to retrieve salt");
            panic!("no salt was provided");
        });
        salt.unwrap()
    };
}

#[cfg(not(test))]
lazy_static! {
    pub static ref CONFIG: Config<'static> = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: SECRET_KEY.as_bytes(),
        ad: &[],
        hash_length: 32,
    };
}

#[cfg(test)]
lazy_static! {
    pub static ref CONFIG: Config<'static> = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 100,
        time_cost: 1,
        lanes: 1,
        thread_mode: ThreadMode::Sequential,
        secret: SECRET_KEY.as_bytes(),
        ad: &[],
        hash_length: 32,
    };
}

pub fn hash_password(password: &str) -> Result<String> {
    let hash = argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &CONFIG)?;
    Ok(hash)
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool> {
    let matches = argon2::verify_encoded_ext(
        &hashed_password,
        password.as_bytes(),
        SECRET_KEY.as_bytes(),
        &[],
    )?;
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_up_env_vars() {
        use std::env::set_var;
        set_var("SECRET_KEY", "TEST KEY");
        set_var("SALT", "TEST SALT");
    }

    #[test]
    fn hash_password_works() {
        set_up_env_vars();
        let password = String::from("SUPERsecretPasSword1234");
        let hashed_password = hash_password(&password).expect("hash failed");
        let matches = argon2::verify_encoded_ext(
            &hashed_password,
            password.as_bytes(),
            SECRET_KEY.as_bytes(),
            &[],
        )
        .unwrap();
        assert!(matches);
    }

    #[test]
    fn verify_password_works() {
        set_up_env_vars();
        let password = String::from("SUPERsecretPasSword1234");
        let hashed_password = hash_password(&password).expect("hash failed");
        assert!(verify_password(&password, &hashed_password).unwrap());
    }
}
