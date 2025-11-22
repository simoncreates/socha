use log::info;

use crate::{
    i_client_handler::ComCancelHandler,
    internal::{GameResult, GameState, PreparedRoom},
    neutral::Move,
};

// todo: add on error
pub trait IClientHandler {
    /// called when a move was requested by the server
    /// optionally cancel_handler can be used for checking,
    /// if the move-calculation should be canceled due to the 2 second time limit
    fn calculate_move(&mut self, cancel_handler: ComCancelHandler) -> Move;

    /// called when the server sends a game_state_update
    fn on_gamestate_update(&mut self, _state: GameState);

    /// called when the client has successfully joined a room
    fn on_game_joined(&mut self, room_id: &str) {
        info!("joined game with id: {}", room_id);
    }

    /// called when client left the room
    fn on_game_left(&mut self) {
        info!("exiting since client is not in the game-room anymore");
    }

    /// called when the result of the current game has been received
    fn on_game_result(&mut self, _res: &GameResult) {
        info!("received the result of the current game");
        info!("exiting since the game has ended");
    }

    /// called when the welcome message was received from the server
    fn on_welcome_message(&mut self) {
        info!("received welcome")
    }

    /// is ran, while the enemy is calculating their move
    /// if cancel_handler.is_canceled() returns true, the while waiting function should return as soon as possible
    #[allow(unused_variables)]
    fn while_waiting(&mut self, cancel_handler: ComCancelHandler) {}

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
