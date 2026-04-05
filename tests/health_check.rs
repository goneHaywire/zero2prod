use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
    startup,
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    // arrange
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();

    // act
    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to send request");

    // assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // arrange
    let TestApp { address, .. } = spawn_app().await;

    let client = reqwest::Client::new();

    // act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    // assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    // arrange
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula%40gmail.com", "missing the name"),
        ("", "missing both email and name"),
    ];

    for (invalid_body, error_message) in test_cases {
        // act
        let response = client
            .post(format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request");

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "API didn't fail with 400 when payload is {error_message}"
        );
    }
}

async fn spawn_app() -> TestApp {
    // create listener with a random addr
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration =
        configuration::get_configuration().expect("Failed to get configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    // pass the listener to run to init server
    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");

    tokio::spawn(server);

    // return TestApp
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::query(&format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .execute(&mut connection)
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
