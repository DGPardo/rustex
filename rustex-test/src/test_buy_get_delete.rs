use std::env;

use dotenvy::dotenv;
use rand::Rng;
use reqwest::{Client, StatusCode};
use serde_json::json;

pub async fn test_buy_get_delete() {
    dotenv().ok(); // Load environment variables from `.env`

    // Load TLS paths
    let cert_path = env::var("TLS_CERT_PATH").expect("TLS_CERT_PATH not set");
    let key_path = env::var("TLS_KEY_PATH").expect("TLS_KEY_PATH not set");
    let ca_path = env::var("TLS_CA_PATH");

    if ca_path.is_ok() {
        println!(
            "\nUsing the following certificates:\n  - Certificate: {}\n  - Key: {}\n  - CA: {}\n",
            cert_path, key_path, ca_path.as_ref().unwrap()
        );
    } else {
        println!(
            "\nUsing the following certificates:\n  - Certificate: {}\n  - Key: {}",
            cert_path, key_path
        );
        println!("\n\n    [WARNING] danger_accept_invalid_certs = true\n\n")
    }

    let cert_content = std::fs::read(cert_path).expect("Failed to read TLS certificate");
    let key_content = std::fs::read(key_path).expect("Failed to read TLS key");
    let ca_cert = ca_path.map(|ca_path| std::fs::read(ca_path).expect("Failed to read CA certificate"));

    let mut client_builder = Client::builder()
        .identity(
            reqwest::tls::Identity::from_pkcs8_pem(&cert_content, &key_content)
                .expect("Invalid client cert"),
        );
    if let Ok(ca_cert) = ca_cert {
        client_builder = client_builder.add_root_certificate(reqwest::tls::Certificate::from_pem(&ca_cert).expect("Invalid CA cert"));
    } else {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    let client = client_builder.build().expect("Failed to create HTTP client");

    let api_base_url = std::env::var("RUSTEX_API_URL").unwrap();

    // Step 1: Login and get bearer token
    let login_url = format!("{}/v1/public/auth/login", api_base_url);
    let login_response = client
        .post(&login_url)
        .basic_auth("foo", Some("bar"))
        .send()
        .await
        .expect("Failed to send login request");

    assert!(
        login_response.status().is_success(),
        "Failed to get bearer token"
    );
    let bearer_token = login_response.text().await.expect("Failed to read token");

    let mut rng = rand::rng();
    let quantity: f64 = rng.random_range(0.1..1_000_000.0);

    // Step 2: Create a new order
    let orders_url = format!("{}/v1/orders", api_base_url);
    let order_response = client
        .post(&orders_url)
        .json(&json!({
            "price": 1,
            "quantity": quantity,
            "exchange": "BTC_USD",
            "orderType": "buy"
        }))
        .header("Authorization", bearer_token.clone())
        .header("Content-Type", "application/json".to_string())
        .send()
        .await
        .expect("Failed to execute buy transaction");

    assert_eq!(
        order_response.status(),
        StatusCode::OK,
        "Failed to create order"
    );
    let order_id = order_response
        .text()
        .await
        .expect("Failed to read order ID");

    // Step 3: Get all user orders
    let all_orders_response = client
        .get(&orders_url)
        .header("Authorization", bearer_token.clone())
        .header("Content-Type", "application/json".to_string())
        .send()
        .await
        .expect("Failed to retrieve my orders");

    assert!(
        all_orders_response.status().is_success(),
        "Failed to get user orders"
    );
    let all_orders_text = all_orders_response
        .text()
        .await
        .expect("Failed to parse orders response");
    assert!(
        all_orders_text.contains("BTC_USD") && all_orders_text.contains(&order_id),
        "Order ID not found in order list"
    );

    // Step 4: Check if order is pending
    let order_status_url = format!("{}/v1/BTC_USD/{}", api_base_url, order_id);
    let order_status_response = client
        .get(&order_status_url)
        .header("Authorization", bearer_token.clone())
        .header("Content-Type", "application/json".to_string())
        .send()
        .await
        .expect("Failed to get order state");

    assert!(
        order_status_response.status().is_success(),
        "Failed to check order state"
    );
    let (is_pending, _remaining): (bool, f64) = order_status_response
        .json()
        .await
        .expect("Failed to parse order state");
    assert!(is_pending, "Order is not pending");

    // Step 5: Delete order
    let delete_response = client
        .delete(&order_status_url)
        .header("Authorization", bearer_token.clone())
        .send()
        .await
        .expect("Failed to delete order");

    assert!(
        delete_response.status().is_success(),
        "Failed to delete order"
    );
    let is_deleted: bool = delete_response
        .json()
        .await
        .expect("Failed to parse delete response");
    assert!(is_deleted, "Order could not be deleted");

    // Step 6: Verify order is NOT pending anymore
    let final_status_response = client
        .get(&order_status_url)
        .header("Authorization", bearer_token.clone())
        .header("Content-Type", "application/json".to_string())
        .send()
        .await
        .expect("Failed to get final order state");

    assert!(
        final_status_response.status().is_success(),
        "Failed to get final order state"
    );
    let (is_pending, _remaining): (bool, f64) = final_status_response
        .json()
        .await
        .expect("Failed to parse final order state");
    assert!(!is_pending, "Order is still pending after deletion");

    println!("✅ All tests passed!");
}
