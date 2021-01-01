use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CustomerData {
    // Shouldn't ever be publically available
    pub private_id: Option<Uuid>,
    pub public_id: Option<Uuid>,
    pub cart_id: Option<Uuid>,
    pub email: Option<String>,
    pub password: Option<String>,
}
