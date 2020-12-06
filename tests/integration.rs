use anyhow::Result;
use assert_json_diff::assert_json_include;
use chrono::DateTime;
use serde_json::json;
use uuid::Uuid;

mod helpers;

use helpers::*;

use bazaar::models::{Customer, ShoppingCart};

const CUSTOMER_GRAPHQL_FIELDS: &str = "#
id,
firstName,
lastName,
email,
createdAt,
lastModified
#";

const SHOPPING_CART_GRAPHQL_FIELDS: &str = "#
id
cartType
items {
   sku 
   quantity
   pricePerUnit
   name
   tags
}
priceBeforeDiscounts
discounts
priceAfterDiscounts
currency
lastModified
createdAt
#";

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn mutation_create_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let graphql_mutatation = format!(
        r#"
        mutation createCustomer($email: String!, $firstName: String!, $lastName: String!) {{
            createCustomer(email: $email, firstName: $firstName, lastName: $lastName) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let email = format!("{}@test.com", Uuid::new_v4());
    let first_name = Uuid::new_v4();
    let last_name = Uuid::new_v4();

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name
        }
    });
    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["createCustomer"].clone();

    assert_json_include!(
        actual: data,
        expected: json!({
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
        })
    );
    Ok(())
}

#[actix_rt::test]
async fn query_customer_by_id_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let customer_id = insert_default_customer(&app.db_pool)
        .await?
        .customer
        .unwrap();

    let graphql_mutatation = format!(
        r#"
        query customerById($id: UUID!) {{
            customerById(id: $id) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "id": customer_id,
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["customerById"].clone();
    assert_json_include!(actual: data, expected: DEFAULT_CUSTOMER.clone());
    Ok(())
}

#[actix_rt::test]
async fn query_customer_by_email_works() -> Result<(), Box<dyn std::error::Error>> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    insert_default_customer(&app.db_pool)
        .await
        .expect("default customer failed to be created");

    let graphql_mutatation = format!(
        r#"
        query customerByEmail($email: String!) {{
            customerByEmail(email: $email) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": format!("{}@test.com", Uuid::nil()),
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["customerByEmail"].clone();
    assert_json_include!(actual: data, expected: DEFAULT_CUSTOMER.clone());
    Ok(())
}

#[actix_rt::test]
async fn mutation_update_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let customer_id = insert_default_customer(&app.db_pool)
        .await?
        .customer
        .unwrap();

    let graphql_mutatation = format!(
        r#"
        mutation updateCustomer($id: UUID!, $update: CustomerUpdate) {{
            updateCustomer(id: $id, update: $update) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "id": customer_id,
            "update": {
                "email": "updated@test.com",
                "firstName": "updated",
                "lastName": "updated"
            }
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["updateCustomer"].clone();
    dbg!(&data);

    assert_json_include!(
        actual: &data,
        expected: json!({
            "firstName": "updated",
            "lastName": "updated",
            "email": "updated@test.com"
        })
    );

    let last_modified = DateTime::parse_from_rfc3339(&data["lastModified"].as_str().unwrap())
        .expect("date should parse correctly with rfc3339");
    let created_at = DateTime::parse_from_rfc3339(&data["createdAt"].as_str().unwrap())
        .expect("date should parse correctly with rfc3339");

    assert!(last_modified > created_at);
    Ok(())
}

#[actix_rt::test]
async fn mutation_create_anonymous_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let graphql_mutatation = format!(
        r#"
        mutation createAnonymousCart($currency: Currency!) {{
            createAnonymousCart(currency: $currency) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "currency": "GBP",
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["createAnonymousCart"].clone();
    assert_json_include!(
        actual: data,
        expected: json!({
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": []
        })
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_create_known_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let customer = insert_default_customer(&app.db_pool)
        .await?
        .customer
        .unwrap();

    let graphql_mutatation = format!(
        r#"
        mutation createKnownCart($id: UUID!, $currency: Currency!) {{
            createKnownCart(id: $id, currency: $currency) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "currency": "GBP",
            "id": customer
        }
    });

    dbg!(&body);
    let response = client.post(&app.address).json(&body).send().await?;
    dbg!(&response);
    let data = response.json::<serde_json::Value>().await?["data"]["createKnownCart"].clone();
    assert_json_include!(
        actual: &data,
        expected: json!({
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": []
        })
    );

    let customer = Customer::find_by_id(customer, &app.db_pool)
        .await
        .expect("failed to query customer from the database")
        .expect("failed to find customer");
    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        Uuid::parse_str(&data["id"].as_str().expect("Cart should always have an ID"))
            .expect("cart id should be valid UUID")
    );
    Ok(())
}

#[actix_rt::test]
async fn mutation_create_known_cart_doesnt_recreate_existing_cart() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation createKnownCart($id: UUID!, $currency: Currency!) {{
            createKnownCart(id: $id, currency: $currency) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "currency": "GBP",
            "id": ids.customer.unwrap()
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;

    dbg!(&response);
    let data = response.json::<serde_json::Value>().await?["data"]["createKnownCart"].clone();
    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": []
        })
    );

    let customer = Customer::find_by_id(ids.customer.unwrap(), &app.db_pool)
        .await
        .expect("failed to query customer from the database")
        .expect("failed to find customer");

    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        Uuid::parse_str(&data["id"].as_str().expect("Cart should always have an ID"))
            .expect("cart id should be valid UUID")
    );
    Ok(())
}

#[actix_rt::test]
async fn mutation_add_item_to_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation addItemsToCart($id: UUID!, $updatedItems: [UpdateCartItem!]!) {{
            addItemsToCart(id: $id, updatedItems: $updatedItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "id": ids.cart.unwrap(),
            "updatedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    dbg!(&body);
    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["addItemsToCart"].clone();
    dbg!(&data);

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [{
                "sku": "12345678",
                "quantity": 3,
                "name": "Item 1",
                "tags": []
            }],
        })
    );
    assert_on_decimal(data["priceBeforeDiscounts"].as_f64().unwrap(), 2.97);
    assert_on_decimal(data["priceAfterDiscounts"].as_f64().unwrap(), 2.97);
    assert_on_decimal(data["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);

    let cart = ShoppingCart::find_by_id(ids.cart.unwrap(), &app.db_pool)
        .await
        .expect("should be able to fetch cart");
    dbg!(&cart);
    assert_eq!(cart.items.len(), 1);
    assert_on_decimal(cart.price_before_discounts, 2.97);
    Ok(())
}
