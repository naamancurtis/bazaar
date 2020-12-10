use anyhow::Result;
use argon2::{self, Config, ThreadMode, Variant, Version};
use lazy_static::lazy_static;
use std::env::var;
use tracing::warn;

lazy_static! {
    pub static ref SECRET_KEY: String = var("SECRET_KEY").expect("no secret key found");
    pub static ref SALT: String = var("SALT").expect("no salt found");
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
    warn!("attempting to hash password now");
    let hash = argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &CONFIG)?;
    warn!("returning hashed pw now");
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
