use actix::{Actor, Addr, ArbiterHandle, Context, Running, StreamHandler};
use actix_web_actors::ws;
use actix_web::{get, post, web, HttpResponse, Responder, middleware, HttpRequest, Error};


/// Define HTTP actor
struct WebsocketActor;

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebsocketActor started");
    }
    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("WebsocketActor stopped")
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebsocketActor {}, &req, stream);
    println!("{:?}", resp);
    resp
}