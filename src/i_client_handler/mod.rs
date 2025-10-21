use std::process::exit;

use log::info;

use crate::{
    internal::{GameResult, GameState, PreparedRoom},
    neutral::Move,
};

// todo: add on error
pub trait IClientHandler {
    fn calculate_move(&mut self) -> Move;

    /// called when the server sends a game_state_update
    fn on_gamestate_update(&mut self, _state: GameState);

    /// called when the client has successfully joined a room
    fn on_game_joined(&mut self, room_id: &str) {
        info!("joined game with id: {}", room_id);
    }

    /// called when client left the room
    fn on_game_left(&mut self) {
        info!("exiting since client is not in the game-room anymore");
        exit(0)
    }

    /// called when the result of the current game has been received
    fn on_game_result(&mut self, _res: &GameResult) {
        info!("received the result of the current game");
        info!("exiting since the game has ended");
        exit(0)
    }

    /// called when the welcome message was received from the server
    fn on_welcome_message(&mut self) {
        info!("received welcome")
    }

    /// is ran while waiting inbetween trying to receive a new message from the server.
    /// when implementing this function, make sure to make its execution non-blocking,
    /// since it wont be killed, when a new message has been received from the server
    fn while_waiting(&mut self) {}

    /// ADMIN
    /// called when a game is prepared
    fn on_game_prepared(&mut self, _prepared: &PreparedRoom) {}

    /// ADMIN
    /// when a game was created
    fn on_create_game(&mut self) {}

    /// ADMIN
    /// called when client starts observing a room
    fn on_observed(&mut self, _room_id: &str) {}
}

#[derive(Debug, Clone)]
pub enum SendCommnad {
    Move(Move),
    SendRaw { xml: String },

    Admin(SendAdminCommand),
}
#[derive(Debug, Clone)]
pub enum SendAdminCommand {
    Authenticate {
        pass: String,
    },
    Observe {
        room_id: String,
    },
    Pause {
        room_id: String,
        pause: bool,
    },
    /// step a pause room forward by one move
    Step {
        room_id: String,
    },
    Cancel {
        room_id: String,
    },
    /// prepares a new room
    Prepare {
        pause: bool,
    },
}
