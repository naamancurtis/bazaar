use anyhow::Result;
use assert_json_diff::assert_json_include;
use chrono::DateTime;
use serde_json::json;
use uuid::Uuid;

use bazaar::{
    auth::decode_token,
    database::{CartItemDatabase, ShoppingCartDatabase},
    models::{cart_item::InternalCartItem, ShoppingCart, TokenType},
};

mod helpers;
use helpers::*;

#[actix_rt::test]
async fn mutation_sign_up_without_token_works() -> Result<()> {
    let app = spawn_app().await;

    let graphql_mutatation = format!(
        r#"
        mutation signUp($email: String!, $password: String!, $firstName: String!, $lastName: String!) {{
            signUp(email: $email, password: $password, firstName: $firstName, lastName: $lastName) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS
    );

    let email = "007@test.com";
    let first_name = "James";
    let last_name = "Bond";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = send_request(&app.address, None, body).await?;
    let tokens = response["data"]["signUp"].clone();

    let issued_at = &tokens["issuedAt"];
    let access_token = &tokens["accessToken"];
    let refresh_token = &tokens["refreshToken"];
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert!(
        access_token
            .as_str()
            .expect("should have valid access token")
            .len()
            > 20
    );
    assert!(
        refresh_token
            .as_str()
            .expect("should have valid refresh token")
            .len()
            > 20
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_sign_up_with_anonymous_token_works() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_anonymous_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation signUp($email: String!, $password: String!, $firstName: String!, $lastName: String!) {{
            signUp(email: $email, password: $password, firstName: $firstName, lastName: $lastName) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS
    );

    let email = "007@test.com";
    let first_name = "James";
    let last_name = "Bond";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let returned_tokens = response["data"]["signUp"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    let access_token = &returned_tokens["accessToken"]
        .as_str()
        .expect("should have valid access token");
    let refresh_token = &returned_tokens["refreshToken"]
        .as_str()
        .expect("should have valid refresh_token");
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert!(access_token.len() > 20);
    assert!(refresh_token.len() > 20);
    let access_token = decode_token(access_token, TokenType::Access)?;
    let refresh_token = decode_token(refresh_token, TokenType::Refresh(0))?;
    assert_eq!(access_token.claims.cart_id, refresh_token.claims.cart_id);
    assert_eq!(access_token.claims.cart_id, tokens.cart_id);

    Ok(())
}

#[actix_rt::test]
async fn mutation_sign_up_with_known_tokens_should_error() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_known_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation signUp($email: String!, $password: String!, $firstName: String!, $lastName: String!) {{
            signUp(email: $email, password: $password, firstName: $firstName, lastName: $lastName) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS
    );

    let email = "007@test.com";
    let first_name = "James";
    let last_name = "Bond";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let errors = response["errors"].clone();
    assert_json_include!(
        actual: errors,
        expected: json!([{
            "message": "Bad Request: Customer already exists",
            "extensions": {
                "status": 400,
                "statusText": "BAD_REQUEST"
            }
        }])
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_valid_credentials_and_no_tokens_works() -> Result<()> {
    let app = spawn_app().await;
    let customer_details = insert_default_customer(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": customer_details.email.clone().unwrap(),
            "password": customer_details.password.unwrap()
        }
    });

    let response = send_request(&app.address, None, body).await?;
    let returned_tokens = response["data"]["login"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    let access_token = &returned_tokens["accessToken"]
        .as_str()
        .expect("should have valid access token");
    let refresh_token = &returned_tokens["refreshToken"]
        .as_str()
        .expect("should have valid refresh_token");
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert!(access_token.len() > 20);
    assert!(refresh_token.len() > 20);
    let access_token = decode_token(access_token, TokenType::Access)?;
    let refresh_token = decode_token(refresh_token, TokenType::Refresh(0))?;
    assert_eq!(access_token.claims.cart_id, refresh_token.claims.cart_id);
    assert_eq!(
        access_token.claims.cart_id,
        customer_details.cart_id.unwrap()
    );

    Ok(())
}

// @TODO need to verify that the carts are merged correctly
#[actix_rt::test]
async fn mutation_login_with_valid_credentials_and_anonymous_tokens_works() -> Result<()> {
    let app = spawn_app().await;
    let customer_details = insert_default_customer(&app.db_pool).await?;
    let tokens = get_anonymous_token(&app.db_pool).await?;
    assert_ne!(tokens.cart_id, customer_details.cart_id.clone().unwrap());

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": customer_details.email.clone().unwrap(),
            "password": customer_details.password.unwrap()
        }
    });

    let response = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let returned_tokens = response["data"]["login"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    let access_token = &returned_tokens["accessToken"]
        .as_str()
        .expect("should have valid access token");
    let refresh_token = &returned_tokens["refreshToken"]
        .as_str()
        .expect("should have valid refresh_token");
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert!(access_token.len() > 20);
    assert!(refresh_token.len() > 20);
    let access_token = decode_token(access_token, TokenType::Access)?;
    let refresh_token = decode_token(refresh_token, TokenType::Refresh(0))?;
    assert_eq!(access_token.claims.cart_id, refresh_token.claims.cart_id);
    assert_eq!(
        access_token.claims.cart_id,
        customer_details.cart_id.unwrap()
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_already_logged_in_customer_errors() -> Result<()> {
    let app = spawn_app().await;
    let customer_details = get_known_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": "random@email.com",
            "password": "randomPassw0rd!"
        }
    });

    let response = send_request(
        &app.address,
        Some(&customer_details.tokens.access_token),
        body,
    )
    .await?;
    let errors = response["errors"].clone();

    assert_json_include!(
        actual: errors,
        expected: json!([{
            "message": "Bad Request: Customer already has valid tokens",
            "extensions": {
                "status": 400,
                "statusText": "BAD_REQUEST"
            }
        }])
    );
    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_unknown_customer_errors() -> Result<()> {
    let app = spawn_app().await;
    let customer_details = insert_default_customer(&app.db_pool).await?;
    let tokens = get_anonymous_token(&app.db_pool).await?;
    assert_ne!(tokens.cart_id, customer_details.cart_id.clone().unwrap());

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": "random@email.com",
            "password": "randomPassw0rd!"
        }
    });

    // [ No tokens | Anonymous Tokens ]
    let test_cases = vec![None, Some(tokens.tokens.access_token.as_str())];

    for case in test_cases.into_iter() {
        let response = send_request(&app.address, case, body.clone()).await?;
        let errors = response["errors"].clone();

        assert_json_include!(
            actual: errors,
            expected: json!([{
                "message": "Could not find resource",
                "extensions": {
                    "status": 404,
                    "statusText": "NOT_FOUND"
                }
            }])
        );
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_invalid_credentials_errors() -> Result<()> {
    let app = spawn_app().await;
    let customer_details = insert_default_customer(&app.db_pool).await?;
    let tokens = get_anonymous_token(&app.db_pool).await?;
    assert_ne!(tokens.cart_id, customer_details.cart_id.clone().unwrap());

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": "imbatman@test.com",
            "password": "randomPassw0rd!"
        }
    });

    // [ No tokens | Anonymous Tokens ]
    let test_cases = vec![None, Some(tokens.tokens.access_token.as_str())];

    for case in test_cases.into_iter() {
        let response = send_request(&app.address, case, body.clone()).await?;
        let errors = response["errors"].clone();

        assert_json_include!(
            actual: errors,
            expected: json!([{
                "message": "Incorrect credentials provided",
                "extensions": {
                    "status": 401,
                    "statusText": "UNAUTHORIZED"
                }
            }])
        );
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_anonymous_login_works() -> Result<()> {
    let app = spawn_app().await;

    let graphql_mutatation = format!(
        r#"
        mutation anonymousLogin {{
            anonymousLogin{{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = send_request(&app.address, None, body.clone()).await?;
    let returned_tokens = response["data"]["anonymousLogin"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    let access_token = &returned_tokens["accessToken"]
        .as_str()
        .expect("should have valid access token");
    let refresh_token = &returned_tokens["refreshToken"]
        .as_str()
        .expect("should have valid refresh_token");
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert!(access_token.len() > 20);
    assert!(refresh_token.len() > 20);
    let access_token = decode_token(access_token, TokenType::Access)?;
    let refresh_token = decode_token(refresh_token, TokenType::Refresh(0))?;
    assert_eq!(access_token.claims.cart_id, refresh_token.claims.cart_id);

    Ok(())
}

#[actix_rt::test]
async fn mutation_update_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_known_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation updateCustomer($update: [CustomerUpdate!]!) {{
            updateCustomer(update: $update) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let generate_json = |key: &str, value: &str| -> serde_json::Value {
        json!({
            "key": key,
            "value": value
        })
    };

    let test_cases = vec![
        json!([generate_json("email", "updated@test.com")]),
        json!([generate_json("firstName", "Updated")]),
        json!([generate_json("lastName", "Updated")]),
        json!([
            generate_json("email", "deadpool@troll.com"),
            generate_json("firstName", "Mr"),
            generate_json("lastName", "Pool")
        ]),
    ];
    let expected = vec![
        json!({
            "firstName": "Bruce",
            "lastName": "Wayne",
            "email": "updated@test.com"
        }),
        json!({
            "firstName": "Updated",
            "lastName": "Wayne",
            "email": "updated@test.com"
        }),
        json!({
            "firstName": "Updated",
            "lastName": "Updated",
            "email": "updated@test.com"
        }),
        json!({
            "firstName": "Mr",
            "lastName": "Pool",
            "email": "deadpool@troll.com"
        }),
    ];

    for (case, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let body = json!({
            "query": graphql_mutatation,
            "variables": {
                "id": tokens.customer.public_id.clone().unwrap(),
                "update": case
            }
        });
        let response = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
        let data = response["data"]["updateCustomer"].clone();

        assert_json_include!(actual: &data, expected: expected);

        let last_modified = DateTime::parse_from_rfc3339(&data["lastModified"].as_str().unwrap())
            .expect("date should parse correctly with rfc3339");
        let created_at = DateTime::parse_from_rfc3339(&data["createdAt"].as_str().unwrap())
            .expect("date should parse correctly with rfc3339");

        assert!(last_modified > created_at);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_update_customer_without_known_token_errors() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_anonymous_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation updateCustomer($update: [CustomerUpdate!]!) {{
            updateCustomer(update: $update) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let update = json!([{
        "key": "firstName",
        "value": "Clark"
    }]);

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "update": update
            }
    });

    // [ No Token | Anonymous Token ]
    let test_cases = vec![None, Some(tokens.tokens.access_token.as_str())];
    let expected = vec![
        json!([{
            "message": "Invalid token provided",
            "extensions": {
                "status": 401,
                "statusText": "INVALID_TOKEN",
                "details": "No token was found"
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

    for (token, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&app.address, token, body.clone()).await?;
        let errors = response["errors"].clone();
        assert_json_include!(actual: errors, expected: expected);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_add_item_to_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let anonymous_tokens = get_anonymous_token(&app.db_pool).await?;
    let customer_details = get_known_token(&app.db_pool).await?;
    assert_ne!(
        anonymous_tokens.cart_id,
        customer_details.customer.cart_id.clone().unwrap()
    );

    let graphql_mutatation = format!(
        r#"
        mutation addItemsToCart($newItems: [UpdateCartItem!]!) {{
            addItemsToCart(newItems: $newItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "newItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let test_cases = vec![
        Some(anonymous_tokens.tokens.access_token.as_str()),
        Some(customer_details.tokens.access_token.as_str()),
    ];

    let expected = vec![
        json!({
            "id": anonymous_tokens.cart_id,
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": [{
                "sku": "12345678",
                "quantity": 3,
                "name": "Item 1",
                "tags": []
            }],
        }),
        json!({
            "id": customer_details.customer.cart_id.clone().unwrap(),
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [{
                "sku": "12345678",
                "quantity": 3,
                "name": "Item 1",
                "tags": []
            }],
        }),
    ];

    for (case, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&app.address, case, body.clone()).await?;
        let cart = response["data"]["addItemsToCart"].clone();

        assert_json_include!(actual: &cart, expected: &expected);
        assert_on_decimal(cart["priceBeforeDiscounts"].as_f64().unwrap(), 2.97);
        assert_on_decimal(cart["priceAfterDiscounts"].as_f64().unwrap(), 2.97);
        assert_on_decimal(cart["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);

        let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(
            Uuid::parse_str(expected["id"].as_str().expect("should have valid UUID"))
                .expect("should be valid UUID"),
            &app.db_pool,
        )
        .await
        .expect("should be able to fetch cart");
        assert_eq!(cart.items.len(), 1);
        assert_on_decimal(cart.price_before_discounts, 2.97);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_item_from_cart_completely_removes_negative_quantities() -> Result<()> {
    let app = spawn_app().await;
    let anonymous_tokens = get_anonymous_token(&app.db_pool).await?;
    let customer_details = get_known_token(&app.db_pool).await?;
    assert_ne!(
        anonymous_tokens.cart_id,
        customer_details.customer.cart_id.clone().unwrap()
    );

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        anonymous_tokens.cart_id,
        vec![InternalCartItem {
            sku: "12345678".to_string(),
            quantity: 1,
        }],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        customer_details.customer.cart_id.clone().unwrap(),
        vec![InternalCartItem {
            sku: "12345678".to_string(),
            quantity: 1,
        }],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(removedItems: $removedItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    // This update would actually set the quantity to -2
    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let test_cases = vec![
        Some(anonymous_tokens.tokens.access_token.as_str()),
        Some(customer_details.tokens.access_token.as_str()),
    ];

    let expected = vec![
        json!({
            "id": anonymous_tokens.cart_id,
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": [],
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0
        }),
        json!({
            "id": customer_details.customer.cart_id.clone().unwrap(),
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [],
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0
        }),
    ];

    for (case, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&app.address, case, body.clone()).await?;
        let cart = response["data"]["removeItemsFromCart"].clone();

        assert_json_include!(actual: &cart, expected: &expected);

        let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(
            Uuid::parse_str(expected["id"].as_str().expect("should have valid UUID"))
                .expect("should be valid UUID"),
            &app.db_pool,
        )
        .await
        .expect("should be able to fetch cart");
        assert!(cart.items.is_empty());
        assert!(cart.price_after_discounts == 0f64);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_items_from_cart_correctly_handles_leftover_items() -> Result<()> {
    let app = spawn_app().await;
    let anonymous_tokens = get_anonymous_token(&app.db_pool).await?;
    let customer_details = get_known_token(&app.db_pool).await?;
    assert_ne!(
        anonymous_tokens.cart_id,
        customer_details.customer.cart_id.clone().unwrap()
    );

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        anonymous_tokens.cart_id,
        vec![
            InternalCartItem {
                sku: "12345678".to_string(),
                quantity: 5,
            },
            InternalCartItem {
                sku: "22345678".to_string(),
                quantity: 2,
            },
        ],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        customer_details.customer.cart_id.clone().unwrap(),
        vec![
            InternalCartItem {
                sku: "12345678".to_string(),
                quantity: 5,
            },
            InternalCartItem {
                sku: "22345678".to_string(),
                quantity: 2,
            },
        ],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(removedItems: $removedItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let test_cases = vec![
        Some(anonymous_tokens.tokens.access_token.as_str()),
        Some(customer_details.tokens.access_token.as_str()),
    ];

    let expected = vec![
        json!({
            "id": anonymous_tokens.cart_id,
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": [
                {
                    "sku": "12345678",
                    "quantity": 2,
                    "name": "Item 1",
                    "tags": []
                },
                {
                    "sku": "22345678",
                    "quantity": 2,
                    "name": "Item 2",
                    "tags": []
                }
            ],
        }),
        json!({
            "id": customer_details.customer.cart_id.clone().unwrap(),
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [
                {
                    "sku": "12345678",
                    "quantity": 2,
                    "name": "Item 1",
                    "tags": []
                },
                {
                    "sku": "22345678",
                    "quantity": 2,
                    "name": "Item 2",
                    "tags": []
                }
            ],
        }),
    ];

    for (case, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&app.address, case, body.clone()).await?;
        let cart = response["data"]["removeItemsFromCart"].clone();

        assert_json_include!(actual: &cart, expected: &expected);
        assert_on_decimal(cart["priceBeforeDiscounts"].as_f64().unwrap(), 22.98);
        assert_on_decimal(cart["priceAfterDiscounts"].as_f64().unwrap(), 22.98);
        assert_on_decimal(cart["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);
        assert_on_decimal(cart["items"][1]["pricePerUnit"].as_f64().unwrap(), 10.50);

        let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(
            Uuid::parse_str(expected["id"].as_str().expect("should have valid UUID"))
                .expect("should be valid UUID"),
            &app.db_pool,
        )
        .await
        .expect("should be able to fetch cart");
        assert_eq!(cart.items.len(), 2);
        assert!(cart.price_before_discounts < 23.0);
    }

    Ok(())
}
