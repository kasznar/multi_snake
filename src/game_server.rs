use std::time::Duration;
use actix::{Actor, Context, Handler, MessageResult};
use serde::{Serialize};
use actix::prelude::*;
use crate::game::Game;

const UPDATE_RATE: Duration = Duration::from_millis(1000);

#[derive(Serialize)]
pub enum GameConnectResult {
    Player1,
    Player2,
    GameFull,
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

pub struct GameServer {
    player1: Option<Recipient<GameUpdated>>,
    player2: Option<Recipient<GameUpdated>>,
    game: Game,
    handle: Option<SpawnHandle>,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer{
            player1: None,
            player2: None,
            handle: None,
            game: Game::new(),
        }
    }

    fn start_game(&mut self, ctx: &mut Context<Self>) {
        let handle = ctx.run_interval(UPDATE_RATE, |act, ctx| {
            println!("Running game update");
            act.game.tick();
            let state = &act.game;

            if let Some(player1) = &act.player1 {
                player1.do_send(GameUpdated { state: state.clone() });
            }

            if let Some(player2) = &act.player2 {
                player2.do_send(GameUpdated { state: state.clone() });
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

        if self.player1 == None {
            self.player1 = Some(msg.address);
            return MessageResult(GameConnectResult::Player1)
        }

        if self.player2 == None {
            self.player2 = Some(msg.address);
            self.start_game(ctx);
            return MessageResult(GameConnectResult::Player2)
        }

        MessageResult(GameConnectResult::GameFull)
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

