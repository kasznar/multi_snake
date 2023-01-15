use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::Duration;
use actix::{Actor, Context, Handler, MessageResult};
use serde::{Serialize};
use actix::prelude::*;
use crate::game::{Direction, Game, Player};
use rand::{self, rngs::ThreadRng, Rng};
use crate::{game_session, session};

pub struct ConnectGameServerResult {
    pub session_id: usize,
}

pub struct ConnectGameServer {
    pub session_address: Addr<session::WsClientSession>,
    pub game_session_id: Option<usize>,
}

impl actix::Message for ConnectGameServer {
    type Result = ConnectGameServerResult;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopGame {
    pub game_session_id: usize,
}

pub struct GameSessionItem {
    address: Addr<game_session::GameSession>,
    id: usize,
}

pub struct GameServer {
    game_sessions: HashMap<usize, GameSessionItem>,
    rng: ThreadRng,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer {
            game_sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<ConnectGameServer> for GameServer {
    type Result = MessageResult<ConnectGameServer>;

    fn handle(&mut self, msg: ConnectGameServer, ctx: &mut Context<Self>) -> Self::Result {
        let game_session_id = msg.game_session_id.unwrap_or_else(|| self.rng.gen::<usize>());

        let game_session = self.game_sessions.entry(game_session_id).or_insert(
            GameSessionItem {
                address: game_session::GameSession::new(game_session_id).start(),
                id: game_session_id
            }
        );

        let session_id = self.rng.gen::<usize>();

        game_session.address.do_send(game_session::ConnectGameSession {
            session_id,
            address: msg.session_address,
        });

        return MessageResult(ConnectGameServerResult {
            session_id,
        });
    }
}

impl Handler<StopGame> for GameServer {
    type Result = MessageResult<StopGame>;

    fn handle(&mut self, msg: StopGame, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(game_session) = &self.game_sessions.get(&msg.game_session_id) {
            game_session.address.do_send(game_session::StopGameSession);
            self.game_sessions.remove(&msg.game_session_id);
        }

        MessageResult(())
    }
}


