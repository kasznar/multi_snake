use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::Duration;
use actix::{Actor, Context, Handler, MessageResult};
use serde::{Serialize};
use actix::prelude::*;
use crate::game::{Direction, Game, Player};
use rand::{self, rngs::ThreadRng, Rng};
use crate::session;

const UPDATE_RATE: Duration = Duration::from_millis(1000);

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ConnectGameSessionResult {
    pub success: bool,
    pub game_session_id: usize,
    pub game_session: Addr<GameSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectGameSession {
    pub address: Addr<session::WsClientSession>,
    pub session_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopGameSession;

// todo: tuple type
#[derive(Message)]
#[rtype(result = "()")]
pub struct GameUpdated {
    pub state: Game,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChangeDirection {
    pub session_id: usize,
    pub direction: Direction,
}

struct PlayerSession {
    id: usize,
    address: Addr<session::WsClientSession>,
}

pub struct GameSession {
    id: usize,
    player1: Option<PlayerSession>,
    player2: Option<PlayerSession>,
    game: Game,
    handle: Option<SpawnHandle>,
    rng: ThreadRng,
}

impl GameSession {
    pub fn new(id: usize) -> GameSession {
        GameSession {
            id,
            player1: None,
            player2: None,
            handle: None,
            game: Game::new(),
            rng: rand::thread_rng(),
        }
    }

    fn start_game(&mut self, ctx: &mut Context<Self>) {
        let handle = ctx.run_interval(UPDATE_RATE, |act, ctx| {
            act.game.tick();
            let state = &act.game;


            if let Some(player1) = &act.player1 {
                player1.address.do_send(GameUpdated { state: state.clone() });
            }

            if let Some(player2) = &act.player2 {
                player2.address.do_send(GameUpdated { state: state.clone() });
            }
        });

        self.handle = Some(handle);
    }

    fn stop_game(&mut self, ctx: &mut Context<Self>) {
        if let Some(handle) = self.handle {
            ctx.cancel_future(handle);
        }
        ctx.stop();
    }
}

impl Actor for GameSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // self.start_game(ctx);
        println!("GameSession started");
    }
    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.stop_game(ctx);
        println!("GameSession stopped")
    }
}

impl Handler<ConnectGameSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: ConnectGameSession, ctx: &mut Context<Self>) -> Self::Result {
        if self.player1.is_some() && self.player2.is_some() {
            msg.address.do_send(ConnectGameSessionResult {
                success: false,
                game_session_id: self.id,
                game_session: ctx.address(),
            });

            return;
        }

        let player_session = PlayerSession {
            address: msg.address.clone(),
            id: msg.session_id,
        };

        if self.player1.is_none() {
            self.player1 = Some(player_session);
        } else if self.player2.is_none() {
            self.player2 = Some(player_session);
            self.start_game(ctx);
        }

        msg.address.do_send(ConnectGameSessionResult {
            success: true,
            game_session_id: self.id,
            game_session: ctx.address(),
        });
    }
}

impl Handler<StopGameSession> for GameSession {
    type Result = MessageResult<StopGameSession>;

    fn handle(&mut self, _: StopGameSession, ctx: &mut Context<Self>) -> Self::Result {
        self.stop_game(ctx);

        MessageResult(())
    }
}

impl Handler<ChangeDirection> for GameSession {
    type Result = MessageResult<ChangeDirection>;

    fn handle(&mut self, msg: ChangeDirection, ctx: &mut Self::Context) -> Self::Result {
        if let Some(player1) = &self.player1 {
            if player1.id == msg.session_id {
                self.game.change_direction(Player::Player1, msg.direction);
            }
        }

        if let Some(player2) = &self.player2 {
            if player2.id == msg.session_id {
                self.game.change_direction(Player::Player2, msg.direction);
            }
        }

        MessageResult(())
    }
}

