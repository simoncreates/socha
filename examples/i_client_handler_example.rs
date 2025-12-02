use log::LevelFilter;
use simple_logging::log_to_file;
use socha::{error::ComError, internal::GameState};

use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::start_iclient;

use rand::Rng;

#[derive(Debug, Default)]
pub struct Logic {
    game_state: GameState,
}

impl IClientHandler for Logic {
    fn calculate_move(&mut self) -> socha::neutral::Move {
        println!("move-request erhalten");
        let mut rng = rand::rng();
        let moves = self.game_state.possible_moves();

        moves[rng.random_range(0..moves.len())]
    }

    fn on_gamestate_update(&mut self, state: socha::internal::GameState) {
        self.game_state = state;
    }

    fn while_waiting(&mut self, cancel_handler: socha::i_client_handler::ComCancelHandler) {
        loop {
            if cancel_handler.is_cancelled() {
                break;
            }
        }
    }
}
/// random bot Beispiel
fn main() -> Result<(), ComError> {
    // logs in datei speichern
    log_to_file("com.log", LevelFilter::Info).unwrap();
    let mut handler = Logic::default();
    start_iclient(
        "localhost:13050",
        None,
        &mut handler,
        std::time::Duration::from_millis(2),
        std::time::Duration::from_secs_f64(1.0), // maximale wartzeit für die while_waiting funktion
    )?;
    Ok(())
}
