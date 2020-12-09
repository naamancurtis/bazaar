use anyhow::Result;
use assert_json_diff::assert_json_include;
use chrono::DateTime;
use serde_json::json;
use uuid::Uuid;

use bazaar::{
    database::{CartItemDatabase, CustomerDatabase, ShoppingCartDatabase},
    models::{cart_item::InternalCartItem, Customer, ShoppingCart},
};

mod helpers;
use helpers::*;

#[actix_rt::test]
async fn mutation_create_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let graphql_mutatation = format!(
        r#"
        mutation createCustomer($email: String!, $password: String!, $firstName: String!, $lastName: String!) {{
            createCustomer(email: $email, password: $password, firstName: $firstName, lastName: $lastName) {{
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
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    let customer = &data["createCustomer"];

    assert_json_include!(
        actual: customer,
        expected: json!({
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
        })
    );
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
        mutation updateCustomer($id: UUID!, $update: [CustomerUpdate!]!) {{
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
            "update": [{
                "key": "email",
                "value": "updated@test.com"
            },{
                "key": "firstName",
                "value": "updated"
            }]
        }
    });

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    dbg!(&data);
    let customer = &data["updateCustomer"];

    assert_json_include!(
        actual: &customer,
        expected: json!({
            "firstName": "updated",
            "lastName": Uuid::nil().to_string(),
            "email": "updated@test.com"
        })
    );

    let last_modified = DateTime::parse_from_rfc3339(&customer["lastModified"].as_str().unwrap())
        .expect("date should parse correctly with rfc3339");
    let created_at = DateTime::parse_from_rfc3339(&customer["createdAt"].as_str().unwrap())
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
    let data = parse_graphql_response(response).await?;
    let cart = &data["createAnonymousCart"];

    assert_json_include!(
        actual: cart,
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
        .expect("failed to create default customer");

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

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    let cart = &data["createKnownCart"];

    assert_json_include!(
        actual: &cart,
        expected: json!({
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": []
        })
    );

    let customer = Customer::find_by_id::<CustomerDatabase>(customer, &app.db_pool).await?;
    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        Uuid::parse_str(&cart["id"].as_str().expect("Cart should always have an ID"))
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

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    let cart = &data["createKnownCart"];

    assert_json_include!(
        actual: &cart,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": []
        })
    );

    let customer =
        Customer::find_by_id::<CustomerDatabase>(ids.customer.unwrap(), &app.db_pool).await?;

    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        Uuid::parse_str(&cart["id"].as_str().expect("Cart should always have an ID"))
            .expect("cart id should be valid UUID")
    );
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

#[actix_rt::test]
async fn mutation_add_item_to_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation addItemsToCart($id: UUID!, $newItems: [UpdateCartItem!]!) {{
            addItemsToCart(id: $id, newItems: $newItems) {{
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
            "newItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    let cart = &data["addItemsToCart"];
    dbg!(&cart);

    assert_json_include!(
        actual: &cart,
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
    assert_on_decimal(cart["priceBeforeDiscounts"].as_f64().unwrap(), 2.97);
    assert_on_decimal(cart["priceAfterDiscounts"].as_f64().unwrap(), 2.97);
    assert_on_decimal(cart["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);

    let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(ids.cart.unwrap(), &app.db_pool)
        .await
        .expect("should be able to fetch cart");
    dbg!(&cart);
    assert_eq!(cart.items.len(), 1);
    assert_on_decimal(cart.price_before_discounts, 2.97);
    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_item_from_cart_completely_removes_negative_quantities() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;
    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        ids.cart.unwrap(),
        vec![InternalCartItem {
            sku: "12345678".to_string(),
            quantity: 1,
        }],
        &app.db_pool,
    )
    .await?;
    dbg!(&cart);

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($id: UUID!, $removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(id: $id, removedItems: $removedItems) {{
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
            "id": ids.cart.unwrap(),
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    let cart = &data["removeItemsFromCart"];

    assert_json_include!(
        actual: &cart,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [],
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0
        })
    );

    let cart =
        ShoppingCart::find_by_id::<ShoppingCartDatabase>(ids.cart.unwrap(), &app.db_pool).await?;
    dbg!(&cart);
    assert!(cart.items.is_empty());
    assert!(cart.price_after_discounts == 0f64);
    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_items_from_cart_correctly() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;
    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        ids.cart.unwrap(),
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
    .await
    .expect("should find shopping cart");
    dbg!(&cart);
    assert_eq!(cart.items.len(), 2);
    assert!(cart.price_before_discounts > 22.98);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($id: UUID!, $removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(id: $id, removedItems: $removedItems) {{
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
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let response = client.post(&app.address).json(&body).send().await?;
    let data = parse_graphql_response(response).await?;
    let cart = &data["removeItemsFromCart"];

    assert_json_include!(
        actual: &cart,
        expected: json!({
            "id": ids.cart,
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
        })
    );
    assert_on_decimal(cart["priceBeforeDiscounts"].as_f64().unwrap(), 22.98);
    assert_on_decimal(cart["priceAfterDiscounts"].as_f64().unwrap(), 22.98);
    assert_on_decimal(cart["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);
    assert_on_decimal(cart["items"][1]["pricePerUnit"].as_f64().unwrap(), 10.50);

    let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(ids.cart.unwrap(), &app.db_pool)
        .await
        .expect("should be able to fetch cart");
    dbg!(&cart);
    assert_eq!(cart.items.len(), 2);
    assert!(cart.price_before_discounts < 23.0);
    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_item_from_cart_completely_removes_negative_quantities() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;
    let cart = ShoppingCart::edit_cart_items(
        ids.cart.unwrap(),
        vec![InternalCartItem {
            sku: "12345678".to_string(),
            quantity: 1,
        }],
        &app.db_pool,
    )
    .await
    .expect("should find shopping cart");
    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($id: UUID!, $removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(id: $id, removedItems: $removedItems) {{
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
            "id": ids.cart.unwrap(),
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    dbg!(&body);
    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["removeItemsFromCart"].clone();
    dbg!(&data);

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [],
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0
        })
    );

    let cart = ShoppingCart::find_by_id(ids.cart.unwrap(), &app.db_pool)
        .await
        .expect("should be able to fetch cart");
    dbg!(&cart);
    assert!(cart.items.is_empty());
    assert!(cart.price_after_discounts == 0f64);
    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_items_from_cart_correctly() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;
    let cart = ShoppingCart::edit_cart_items(
        ids.cart.unwrap(),
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
    .await
    .expect("should find shopping cart");
    dbg!(&cart);
    assert_eq!(cart.items.len(), 2);
    assert!(cart.price_before_discounts > 22.98);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($id: UUID!, $removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(id: $id, removedItems: $removedItems) {{
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
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    dbg!(&body);
    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["removeItemsFromCart"].clone();
    dbg!(&data);

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": ids.cart,
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
        })
    );
    assert_on_decimal(data["priceBeforeDiscounts"].as_f64().unwrap(), 22.98);
    assert_on_decimal(data["priceAfterDiscounts"].as_f64().unwrap(), 22.98);
    assert_on_decimal(data["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);
    assert_on_decimal(data["items"][1]["pricePerUnit"].as_f64().unwrap(), 10.50);

    let cart = ShoppingCart::find_by_id(ids.cart.unwrap(), &app.db_pool)
        .await
        .expect("should be able to fetch cart");
    dbg!(&cart);
    assert_eq!(cart.items.len(), 2);
    assert!(cart.price_before_discounts < 23.0);
    Ok(())
}
