use anyhow::Result;
use assert_json_diff::assert_json_include;
use serde_json::json;

mod helpers;
use helpers::*;

#[actix_rt::test]
async fn query_customer_fails_for_an_unknown_user() -> Result<()> {
    let app = spawn_app().await;
    let unauth_client = build_http_client()?;
    let anon_client = build_http_client()?;
    let _customer = get_anonymous_token(&anon_client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        query customer {{
            customer {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });
    let cases = vec![unauth_client, anon_client];
    let expected = vec![
        json!([{
            "message": "Invalid token provided",
            "extensions": {
                "status": 401,
                "statusText": "INVALID_TOKEN"
            }
        }]),
        json!([{
            "message": "Anonymous users do not have access to this resource",
            "extensions": {
                "status": 401,
                "statusText": "UNAUTHORIZED"
            }
        }]),
    ];
    for (client, expected) in cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&client, &app.address, &body).await?;

        let errors = response.data["errors"].clone();
        assert_json_include!(actual: errors, expected: &expected);
    }

    Ok(())
}

#[actix_rt::test]
async fn query_customer_works_for_known_user() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let customer = sign_user_up_and_get_known_token(&client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        query customer {{
            customer {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = send_request(&client, &app.address, &body).await?;
    let data = response.data["data"]["customer"].clone();

    assert_json_include!(
        actual: data,
        expected: json!({
            "id": customer.public_id.unwrap(),
            "firstName": customer.first_name.unwrap(),
            "lastName": customer.last_name.unwrap(),
            "email": customer.email.unwrap(),
        })
    );
    Ok(())
}

#[actix_rt::test]
async fn query_customer_with_known_user_includes_cart() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let customer = sign_user_up_and_get_known_token(&client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        query customer {{
            customer {{
                {}
                cart {{
                    {}
                }}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS, SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = send_request(&client, &app.address, &body).await?;
    let data = response.data["data"]["customer"].clone();
    assert_json_include!(
        actual: data,
        expected: json!({
            "id": customer.public_id.unwrap(),
            "firstName": customer.first_name.unwrap(),
            "lastName": customer.last_name.unwrap(),
            "email": customer.email.unwrap(),
            "cart": {
                "id": customer.cart_id.unwrap(),
                "currency": "GBP",
                "cartType": "KNOWN",
                "priceBeforeDiscounts": 0.0,
                "priceAfterDiscounts": 0.0,
                "items": [],
            }
        })
    );
    Ok(())
}

#[actix_rt::test]
async fn query_cart_works_for_anonymous_user() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let customer = get_anonymous_token(&client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        query cart {{
            cart {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = send_request(&client, &app.address, &body).await?;
    let data = response.data["data"]["cart"].clone();

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": customer.cart_id.unwrap(),
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0,
            "items": [],
        })
    );

    Ok(())
}

#[actix_rt::test]
async fn query_cart_works_for_known_user() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let customer = sign_user_up_and_get_known_token(&client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        query cart {{
            cart {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = send_request(&client, &app.address, &body).await?;
    let data = response.data["data"]["cart"].clone();

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": customer.cart_id.unwrap(),
            "currency": "GBP",
            "cartType": "KNOWN",
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0,
            "items": [],
        })
    );

    Ok(())
}

#[actix_rt::test]
async fn query_health_check_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;

    let body = json!({ "query": "{ healthCheck }" });
    let response = send_request(&client, &app.address, &body).await?;

    let data = response.data["data"]["healthCheck"].clone();
    assert_json_include!(actual: data, expected: true);

    Ok(())
}
