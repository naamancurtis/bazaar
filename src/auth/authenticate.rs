use argon2::{self, Config, ThreadMode, Variant, Version};
use lazy_static::lazy_static;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use sqlx::PgPool;
use std::env::var;
use tracing::error;

use crate::{database::AuthRepository, models::auth::AuthCustomer, BazaarError, Result};

// Ideally, you would not want this as a static variable, as if the server
// is left up and running for a long time, you would want to cycle keys every x
// days and have that appropriately picked up on all running servers.
//
// In reality, to make the above viable, you'd have to integrate it with a key management system,
// as you would need to know what the key was at the time when the user was created, so you could
// correctly fetch the key to validate their password. So for now it will be left as a static
// variable, but for an actual production system with real user data this wouldn't be appropriate
lazy_static! {
    pub static ref SECRET_KEY: String = {
        let secret_key: std::result::Result<String, ()> = var("SECRET_KEY").map_err(|e| {
            error!(err = ?e, "failed to retrieve secret key");
            panic!("no SECRET KEY was provided");
        });
        secret_key.unwrap()
    };
}

#[cfg(not(test))]
lazy_static! {
    pub static ref CONFIG: Config<'static> = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 4096,
        time_cost: 10,
        lanes: 4,
        thread_mode: ThreadMode::Parallel,
        secret: SECRET_KEY.as_bytes(),
        ad: &[],
        hash_length: 256,
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

/// Returns true if the password matches the stored password hash
pub async fn verify_password_and_fetch_details<DB: AuthRepository>(
    email: &str,
    password: &str,
    pool: &PgPool,
) -> Result<AuthCustomer> {
    let customer = DB::get_auth_customer(email, pool).await?;
    if _verify_password(password, &customer.hashed_password)? {
        return Ok(customer);
    }
    Err(BazaarError::IncorrectCredentials)
}

pub fn hash_password(password: &str) -> Result<String> {
    let mut salt = [0u8; 128];
    let mut salt_generator = ChaCha20Rng::from_entropy();
    salt_generator.try_fill_bytes(&mut salt)?;
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &CONFIG)?;
    Ok(hash)
}

fn _verify_password(password: &str, hashed_password: &str) -> Result<bool> {
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
    use async_trait::async_trait;
    use claim::assert_ok;
    use uuid::Uuid;

    use crate::{database::AuthRepository, Result};

    struct MockAuthRepo;

    #[async_trait]
    impl AuthRepository for MockAuthRepo {
        async fn map_id(_: Option<Uuid>, _: &PgPool) -> Result<Option<Uuid>> {
            unimplemented!()
        }
        // Hacky, but just returning the email - so pass the hash through as the email
        async fn get_auth_customer(email: &str, _: &PgPool) -> Result<AuthCustomer> {
            Ok(AuthCustomer {
                id: Uuid::new_v4(),
                public_id: Uuid::new_v4(),
                hashed_password: email.to_string(),
            })
        }
    }

    fn set_up_env_vars() {
        use std::env::set_var;
        set_var("SECRET_KEY", "TEST KEY");
    }

    #[test]
    fn hash_password_works() {
        set_up_env_vars();
        let password = "SUPERsecretPasSword1234";
        let hashed_password = hash_password(password).expect("hash failed");
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
    fn two_identical_passwords_should_have_different_hashes() {
        set_up_env_vars();
        let password = "SUPERsecretPasSword1234";
        let hashed_password_1 = hash_password(password).expect("hash failed");
        let hashed_password_2 = hash_password(password).expect("hash failed");
        assert_ne!(hashed_password_1, hashed_password_2);
    }

    #[tokio::test]
    async fn verify_password_works() {
        set_up_env_vars();
        let password = "SUPERsecretPasSword1234";
        let hashed_password = hash_password(password).expect("hash failed");
        let config = crate::get_configuration().expect("failed to read config");
        let pool = PgPool::connect_lazy(&config.database.raw_pg_url())
            .expect("failed to create fake connection");
        assert_ok!(
            verify_password_and_fetch_details::<MockAuthRepo>(&hashed_password, &password, &pool)
                .await
        );
    }

    #[test]
    fn _verify_password_works() {
        set_up_env_vars();
        let password = "SUPERsecretPasSword1234";
        let hashed_password = hash_password(password).expect("hash failed");
        assert!(_verify_password(password, &hashed_password).unwrap());
    }
}
