use actix::{Actor, Addr, ArbiterHandle, AsyncContext, Context, Running, StreamHandler};
use actix_web::{Error, get, HttpRequest, HttpResponse, middleware, post, Responder, web};
use actix_web_actors::ws;
use actix::prelude::*;
use serde::Serialize;

use crate::game::Direction;
use crate::{game_server, game_session};

pub struct WsClientSession {
    pub id: usize,
    pub game_server: Addr<game_server::GameServer>,
    pub game_session: Option<Addr<game_session::GameSession>>,
    pub game_session_id: Option<usize>,
}

impl Actor for WsClientSession {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<game_session::GameUpdated> for WsClientSession {
    type Result = ();

    fn handle(&mut self, msg: game_session::GameUpdated, ctx: &mut Self::Context) {
        let json = serde_json::to_string(&msg.state);

        if let Ok(text) = json {
            ctx.text(text);
        }
    }
}

#[derive(Serialize)]
struct ConnectGameResponse {
    game_session_id: usize,
}

impl Handler<game_session::ConnectGameSessionResult> for WsClientSession {
    type Result = ();

    fn handle(&mut self, msg: game_session::ConnectGameSessionResult, ctx: &mut Self::Context) -> Self::Result {
        self.game_session = Some(msg.game_session);
        self.game_session_id = Some(msg.game_session_id);

        let response = ConnectGameResponse {
            game_session_id: msg.game_session_id,
        };

        let serde_result = serde_json::to_string(&response);

        if let Ok(text) = serde_result {
            ctx.text(text);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClientSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // todo: how to match directly the ByteString
                let raw_message = text.trim();
                let message: Vec<&str> = raw_message.splitn(2, ' ').collect();

                match message[0] {
                    "/connect" => {
                        let game_session_id = message.get(1).and_then(|id| id.parse::<usize>().ok());

                        self.game_server
                            .send(game_server::ConnectGameServer {
                                session_address: ctx.address(),
                                game_session_id,
                            })
                            .into_actor(self)
                            .then(|res, act, ctx| {
                                match res {
                                    Ok(result) => {
                                        act.id = result.session_id;
                                    }
                                    _ => println!("Something is wrong"),
                                }
                                fut::ready(())
                            })
                            .wait(ctx);
                    }
                    "/stop" => {
                        if let Some(game_session_id) = self.game_session_id {
                            self.game_server.do_send(game_server::StopGame {
                                game_session_id,
                            });
                        }

                    }
                    "/direction" => {
                        let new_direction = match message[1] {
                            "right" => Some(Direction::RIGHT),
                            "left" => Some(Direction::LEFT),
                            "up" => Some(Direction::UP),
                            "down" => Some(Direction::DOWN),
                            _ => None,
                        };

                        if let Some(direction) = new_direction {
                            if let Some(game_session) = &self.game_session {
                                game_session.do_send(game_session::ChangeDirection {
                                    session_id: self.id,
                                    direction,
                                });
                            }
                        }
                    }
                    _ => ctx.text("Unrecognized command")
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}