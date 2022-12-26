use actix::{Actor, Addr, ArbiterHandle, AsyncContext, Context, Running, StreamHandler};
use actix_web::{Error, get, HttpRequest, HttpResponse, middleware, post, Responder, web};
use actix_web_actors::ws;
use actix::prelude::*;
use serde::Serialize;

use crate::game::Direction;
use crate::game_server;

pub struct WsGameSession {
    pub id: usize,
    pub game_server: Addr<game_server::GameServer>,
}

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<game_server::GameUpdated> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: game_server::GameUpdated, ctx: &mut Self::Context) {
        let json = serde_json::to_string(&msg.state);

        if let Ok(text) = json {
            ctx.text(text);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsGameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // todo: how to match directly the ByteString
                let message = text.trim();


                match message {
                    "/connect" => {
                        let address = ctx.address();

                        self.game_server
                            .send(game_server::ConnectGame{
                                address: address.recipient(),
                            })
                            .into_actor(self)
                            .then(|res, act, ctx| {
                                match res {
                                    Ok(result) => {
                                        act.id = result.session_id;

                                        // todo: implement Into<ByteString> for responses
                                        let serde_result = serde_json::to_string(&result.success);
                                        if let Ok(text) = serde_result {
                                            ctx.text(text);
                                        }
                                    }
                                    _ => println!("Something is wrong"),
                                }
                                fut::ready(())
                            })
                            .wait(ctx);
                    }
                    "/stop" => {
                        self.game_server.do_send(game_server::StopGame);
                    }
                    _ => {
                        let new_direction = match message {
                            "right" => Some(Direction::RIGHT),
                            "left" => Some(Direction::LEFT),
                            "up" => Some(Direction::UP),
                            "down" => Some(Direction::DOWN),
                            _ => None,
                        };

                        if let Some(direction) = new_direction {
                            self.game_server.do_send(game_server::ChangeDirection {
                                session_id: self.id,
                                direction
                            });
                        }
                    }
                }
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}