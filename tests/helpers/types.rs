use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CustomerData {
    pub id: Option<Uuid>,
    pub cart_id: Option<Uuid>,
    pub email: Option<String>,
    pub password: Option<String>,
}
