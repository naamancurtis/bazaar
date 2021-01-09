use chrono::Duration;
use lazy_static::lazy_static;

pub const ACCESS_TOKEN_DURATION_SECONDS: i64 = 900;
pub const REFRESH_TOKEN_DURATION_SECONDS: i64 = 2419200;
pub const TOKEN_TYPE: &str = "bearer";

lazy_static! {
    pub static ref TIME_TO_REFRESH: Duration = Duration::days(7);
    pub static ref ACCESS_TOKEN_DURATION: Duration =
        Duration::seconds(ACCESS_TOKEN_DURATION_SECONDS);
    pub static ref REFRESH_TOKEN_DURATION: Duration =
        Duration::seconds(REFRESH_TOKEN_DURATION_SECONDS);
}
