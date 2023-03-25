use std::error;
use std::io;
use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use rs_backend::configuration;
use rs_backend::startup;
use rs_backend::telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    telemetry::init_subscriber(telemetry::get_subscriber("rs_backend", "info", io::stdout));

    // 获取配置
    let config = configuration::get_configuration()?;

    let addr = config.app.addr();
    let dsn = config.database.dsn();

    tracing::info!("listen address: {addr}");

    let connect = PgPool::connect(dsn.expose_secret()).await?;
    let lst = TcpListener::bind(addr)?;
    startup::run(lst, connect)?.await?;
    Ok(())
}
