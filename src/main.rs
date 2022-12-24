use actix_web::{get, post, web, HttpResponse, Responder, middleware, HttpRequest, Error};
use env_logger::Env;
use std::sync::Mutex;
use actix_web_actors::ws;
use actix::{Actor, Addr, ArbiterHandle, Context, Running, StreamHandler};
use crate::examples::AppStateWithCounter;
use actix_files::{Files, NamedFile};

mod examples;
mod websocket;
mod game;


async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    env_logger::init_from_env(Env::default().default_filter_or("info"));


    // Note: web::Data created _outside_ HttpServer::new closure
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        // move counter into the closure
        App::new()
            .app_data(counter.clone())
            .service(examples::echo)
            .route("/hey", web::get().to(examples::manual_hello))
            .route("/count", web::get().to(examples::count))
            .route("/ws/", web::get().to(websocket::index))
            .service(web::resource("/").to(index))
            .service(Files::new("/static", "./static"))
            .wrap(middleware::Logger::default())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}