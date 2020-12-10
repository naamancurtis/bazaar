use anyhow::Result;
use assert_json_diff::assert_json_include;
use serde_json::json;
use uuid::Uuid;

mod helpers;
use helpers::*;

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
async fn query_find_cart_by_id_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        query cartById($id: UUID!) {{
            cartById(id: $id) {{
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
        }
    });

    dbg!(&body);
    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["cartById"].clone();
    dbg!(&data);

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0,
            "items": [],
        })
    );

    Ok(())
}
