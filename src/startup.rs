use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::routers::{health_check, subscribe};

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, Box<dyn error::Error>> {
    let db_pool = web::Data::new(pool);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
