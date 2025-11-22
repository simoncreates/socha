use std::time::Instant;

use log::{info, LevelFilter};
use simple_logging::log_to_file;
use socha::error::ComError;

use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::{start_iclient, ComCancelHandler};
use socha::neutral::Move;

// ___ UNFINISHED ___

#[derive(Debug, Default)]
pub struct Logic {}
impl IClientHandler for Logic {
    fn calculate_move(&mut self, cancel_handler: ComCancelHandler) -> socha::neutral::Move {
        info!("received mvoe");
        let start = Instant::now();
        loop {
            if cancel_handler.is_cancelled() {
                info!("move calculation was canceled");
                break;
            }
        }
        println!("timeout: {:?}", start.elapsed(),);
        Move {
            dir: socha::neutral::Direction::UP,
            from: (0, 0),
        }
    }
    fn on_gamestate_update(&mut self, _state: socha::internal::GameState) {}
}

fn main() -> Result<(), ComError> {
    log_to_file("com.log", LevelFilter::Info).unwrap();
    let mut handler = Logic::default();
    start_iclient(
        "localhost:13050",
        None,
        &mut handler,
        std::time::Duration::from_millis(2),
        std::time::Duration::from_secs_f64(1.9),
    )?;
    Ok(())
}
