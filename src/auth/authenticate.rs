use argon2::{self, Config, ThreadMode, Variant, Version};
use lazy_static::lazy_static;
use sqlx::PgPool;
use std::env::var;
use tracing::error;

use crate::{database::AuthRepository, models::auth::AuthCustomer, BazaarError};

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

/// Returns true if the password matches the stored password hash
pub async fn verify_password_and_fetch_details<DB: AuthRepository>(
    email: &str,
    password: &str,
    pool: &PgPool,
) -> Result<AuthCustomer, BazaarError> {
    let customer = DB::get_auth_customer(email, pool).await?;
    if _verify_password(&password, &customer.password_hash)? {
        return Ok(customer);
    }
    Err(BazaarError::IncorrectCredentials)
}

pub fn hash_password(password: &str) -> Result<String, BazaarError> {
    let hash = argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &CONFIG)?;
    Ok(hash)
}

fn _verify_password(password: &str, hashed_password: &str) -> Result<bool, BazaarError> {
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

    use crate::database::AuthRepository;

    struct MockAuthRepo;

    #[async_trait]
    impl AuthRepository for MockAuthRepo {
        async fn map_id(_: Option<Uuid>, _: &PgPool) -> Option<Uuid> {
            None
        }
        // Hacky, but just returning the email - so pass the hash through as the email
        async fn get_auth_customer(email: &str, _: &PgPool) -> Result<AuthCustomer, BazaarError> {
            Ok(AuthCustomer {
                id: Uuid::new_v4(),
                public_id: Uuid::new_v4(),
                password_hash: email.to_string(),
            })
        }
    }

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

    #[tokio::test]
    async fn verify_password_works() {
        set_up_env_vars();
        let password = String::from("SUPERsecretPasSword1234");
        let hashed_password = hash_password(&password).expect("hash failed");
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
        let password = String::from("SUPERsecretPasSword1234");
        let hashed_password = hash_password(&password).expect("hash failed");
        assert!(_verify_password(&password, &hashed_password).unwrap());
    }
}
