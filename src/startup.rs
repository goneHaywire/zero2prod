use std::net::TcpListener;

use actix_web::{App, HttpServer, dev::Server, web};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscriptions};

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    // wraps connection in an Arc
    let db_pool = web::Data::new(connection_pool);

    // create server and run it
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check::health_check))
            .route("/subscriptions", web::post().to(subscriptions::subscribe))
            // register the connection as appState
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    // return server (future)
    Ok(server)
}
