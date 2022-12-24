use std::time::Duration;
use actix::{Actor, Addr, ArbiterHandle, AsyncContext, Context, Running, StreamHandler};
use actix_web_actors::ws;
use actix_web::{get, post, web, HttpResponse, Responder, middleware, HttpRequest, Error};
use serde::Serialize;
use crate::game::{Direction, Point, Snake};


const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(100);

struct WebsocketActor {
   snake: Snake
}

impl WebsocketActor {
    pub fn new() -> WebsocketActor {
        let snake = Snake::new(Point{x: 10, y: 3});

        WebsocketActor {
            snake
        }
    }

    fn start_game(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            act.snake.update();

            let json = serde_json::to_string(&act.snake);

            if let Ok(text) = json {
                ctx.text(text);
            }
        });
    }
}

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebsocketActor started");
        self.start_game(ctx);
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("WebsocketActor stopped")
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // todo: how to match directly the ByteString
                let message = text.trim();
                
                let new_direction = match message {
                    "right" => Some(Direction::RIGHT),
                    "left" => Some(Direction::LEFT),
                    "up" => Some(Direction::UP),
                    "down" => Some(Direction::DOWN),
                    _ => None,
                };

                if let Some(direction) = new_direction {
                    self.snake.set_direction(direction);
                }


            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebsocketActor::new(), &req, stream);
    println!("{:?}", resp);
    resp
}