use async_graphql::{
    validators::{Email, InputValueValidator},
    Value,
};

pub struct ValidCustomerUpdateType {}

impl InputValueValidator for ValidCustomerUpdateType {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::List(list) => {
                for item in list {
                    let _ = self.is_valid(item)?;
                }
                Ok(())
            }
            Value::Object(obj) => {
                let key = match obj.get("key") {
                    Some(key) => match key {
                        Value::String(key) => match key.as_str() {
                            "firstName" => key,
                            "lastName" => key,
                            "email" => key,
                            invalid_key => return Err(format!("invalid key: {}", invalid_key)),
                        },
                        _ => return Err("invalid object provided".to_string()),
                    },
                    None => return Err("expected object containing key: 'key'".to_string()),
                };

                let _ = match obj.get("value") {
                    Some(value) => match value {
                        Value::String(_) => match key.as_str() {
                            "firstName" => (),
                            "lastName" => (),
                            "email" => {
                                let email = Email {};
                                let _ = email.is_valid(&value)?;
                            }
                            _ => return Err("invalid value passed into update".to_string()),
                        },
                        _ => return Err("invalid value type passed into update".to_string()),
                    },
                    None => return Err("expected object containing key: 'value'".to_string()),
                };
                Ok(())
            }
            _ => Err("invalid input".to_string()),
        }
    }
}
