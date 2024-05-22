use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use tokio::task;
use crate::config::{Config, process_verification_queue};
use logging::init_logger;
use crate::storage::{VERIFY_QUEUE, PLONKY2_HASHMAP};
use crate::routes::{verify_plonky2, verify, ping_single};


mod config;
mod logging;
mod storage;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::init();
    init_logger();
    let queue = VERIFY_QUEUE.clone();
    let plonky2_hashmap = PLONKY2_HASHMAP.clone();
    task::spawn(process_verification_queue(
        queue.clone(),
        plonky2_hashmap.clone(),
    ));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(queue.clone()))
            .app_data(web::Data::new(plonky2_hashmap.clone()))
            .service(verify_plonky2)
            .service(verify)
            .service(ping_single)
    })
    .workers(config.workers)
    .bind(("127.0.0.1", config.port))?
    .run()
    .await
}
