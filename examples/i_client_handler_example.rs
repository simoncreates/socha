use std::time::Instant;

use log::LevelFilter;
use simple_logging::log_to_file;
use socha::error::ComError;

use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::start_iclient;
use socha::neutral::Move;

#[derive(Debug, Default)]
pub struct Logic {}
impl IClientHandler for Logic {
    fn calculate_move(&mut self) -> socha::neutral::Move {
        println!("move-request erhalten");
        let start = Instant::now();
        loop {
            if start.elapsed() > std::time::Duration::from_millis(1900) {
                break;
            }
        }
        Move {
            dir: socha::neutral::Direction::Right,
            from: (0, 2),
        }
    }
    fn on_gamestate_update(&mut self, _state: socha::internal::GameState) {}
    fn while_waiting(&mut self, cancel_handler: socha::i_client_handler::ComCancelHandler) {
        loop {
            if cancel_handler.is_cancelled() {
                println!("warten zuende, der Gegner hat ein Zug gemacht");
                break;
            }
        }
    }
}

fn main() -> Result<(), ComError> {
    log_to_file("com.log", LevelFilter::Info).unwrap();
    let mut handler = Logic::default();
    start_iclient(
        "localhost:13050",
        None,
        &mut handler,
        std::time::Duration::from_millis(2),
        std::time::Duration::from_secs_f64(1.0), // wird für den cancel handler der while waiting function verwendet
    )?;
    Ok(())
}
