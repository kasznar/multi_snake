use std::time::Instant;
use actix::{Actor, Addr};
use actix_files::{Files, NamedFile};
use actix_web::{Error, HttpRequest, HttpResponse, middleware, Responder, web};
use actix_web_actors::ws;
use env_logger::Env;

mod game;
mod game_server;
mod game_session;
mod session;

use actix_cors::Cors;

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

pub async fn game_ws(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<game_server::GameServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(session::WsClientSession {
        id: 0,
        hb: Instant::now(),
        handle: None,
        game_server: server.get_ref().clone(),
        game_session: None,
        game_session_id: None,
    }, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let server = game_server::GameServer::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/ws/", web::get().to(game_ws))
            .service(web::resource("/").to(index))
            .service(Files::new("/static", "./static"))
            .wrap(middleware::Logger::default())
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}