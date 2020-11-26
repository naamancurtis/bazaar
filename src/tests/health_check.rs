#[actix_rt::test]
async fn health_check_works() {
    let addr = spawn_app().expect("failed to spawn app");

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &addr))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = bazar::build_app(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
