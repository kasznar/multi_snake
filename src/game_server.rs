use std::collections::HashMap;
use std::time::Duration;
use actix::{Actor, Context, Handler, MessageResult};
use serde::{Serialize};
use actix::prelude::*;
use crate::game::{Direction, Game, Player};
use rand::{self, rngs::ThreadRng, Rng};

const UPDATE_RATE: Duration = Duration::from_millis(1000);


pub struct GameConnectResult {
    pub success: bool,
    pub session_id: usize,
}

pub struct ConnectGame {
    pub address: Recipient<GameUpdated>,
}

impl actix::Message for ConnectGame {
    type Result = GameConnectResult;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopGame;

// todo: tuple type
#[derive(Message)]
#[rtype(result = "()")]
pub struct GameUpdated{
    pub state: Game,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChangeDirection {
    pub session_id: usize,
    pub direction: Direction
}

struct PlayerSession {
    id: usize,
    address: Recipient<GameUpdated>
}

pub struct GameServer {
    player1: Option<PlayerSession>,
    player2: Option<PlayerSession>,
    game: Game,
    handle: Option<SpawnHandle>,
    rng: ThreadRng,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer{
            player1: None,
            player2: None,
            handle: None,
            game: Game::new(),
            rng: rand::thread_rng(),
        }
    }

    fn start_game(&mut self, ctx: &mut Context<Self>) {
        let handle = ctx.run_interval(UPDATE_RATE, |act, ctx| {
            println!("Running game update");
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
    }
}

impl Actor for GameServer{
    type Context = Context<Self>;
}

impl Handler<ConnectGame> for GameServer {
    type Result = MessageResult<ConnectGame>;

    fn handle(&mut self, msg: ConnectGame, ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        let address = msg.address;

        if self.player1.is_some() && self.player2.is_some() {
            return MessageResult(GameConnectResult {
                session_id: id,
                success:false,
            })
        }

        let player_session = PlayerSession {
            address,
            id,
        };

        if self.player1.is_none() {
            self.player1 = Some(player_session);
        } else if self.player2.is_none() {
            self.player2 = Some(player_session);
            self.start_game(ctx);
        }

        return MessageResult(GameConnectResult {
            session_id: id,
            success: true,
        })
    }
}

impl Handler<StopGame> for GameServer {
    type Result = MessageResult<StopGame>;

    fn handle(&mut self, _: StopGame, ctx: &mut Context<Self>) -> Self::Result {
        self.stop_game(ctx);
        self.player1 = None;
        self.player2 = None;
        self.game = Game::new();

        MessageResult(())
    }
}

impl Handler<ChangeDirection> for GameServer {
    type Result = MessageResult<ChangeDirection>;

    fn handle(&mut self, msg: ChangeDirection, ctx: &mut Self::Context) -> Self::Result {
        if let Some(player1) = &self.player1{
            if player1.id == msg.session_id {
                self.game.change_direction(Player::Player1, msg.direction);
            }
        }

        if let Some(player2) = &self.player2{
            if player2.id == msg.session_id {
                self.game.change_direction(Player::Player2, msg.direction);
            }
        }

        MessageResult(())
    }
}

