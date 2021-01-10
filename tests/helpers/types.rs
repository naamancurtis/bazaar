use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CustomerData {
    // Shouldn't ever be publically available
    pub private_id: Option<Uuid>,
    pub public_id: Option<Uuid>,
    pub cart_id: Option<Uuid>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub raw_access_token: Option<String>,
    pub raw_refresh_token: Option<String>,
}
