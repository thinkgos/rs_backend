use std::{env, error, io, net::TcpListener};

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use rs_backend::{configuration, startup::run, telemetry};

#[tokio::test]
async fn health_check_succeeds() -> Result<(), Box<dyn error::Error>> {
    let app = spawn_app().await?;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/health_check", &app.addr))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), Box<dyn error::Error>> {
    // Arrange
    let app = spawn_app().await?;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("http://{}/subscriptions", &app.addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> Result<(), Box<dyn error::Error>> {
    // Arrange
    let app = spawn_app().await?;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("http://{}/subscriptions", &app.addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }

    Ok(())
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    let subscriber_name = "test";

    if env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

struct TestApp {
    addr: String,
    pg_pool: PgPool,
}

async fn spawn_app() -> Result<TestApp, Box<dyn error::Error>> {
    // let subscriber = telemetry::get_subscriber("test".into(), "debug".into());
    // telemetry::init_subscriber(subscriber);
    Lazy::force(&TRACING);

    let config = configuration::get_configuration()?;
    let ip = "127.0.0.1";
    let dsn = config.database.dsn();

    let lst = TcpListener::bind(format!("{}:0", ip))?;
    let port = lst.local_addr()?.port();

    let pg_pool = PgPool::connect(&dsn.expose_secret()).await.unwrap();
    let server = run(lst, pg_pool.clone()).unwrap();

    tokio::spawn(server);

    Ok(TestApp {
        addr: format!("{}:{}", ip, port),
        pg_pool,
    })
}
