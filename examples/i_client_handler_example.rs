use log::{info, LevelFilter};
use simple_logging::log_to_file;
use socha::i_client_handler::IClientHandler;
use socha::neutral::Move;
use socha::socha_com::{ComError, ComHandler};

// ___ UNFINISHED ___

#[derive(Debug, Default)]
pub struct Logic {}
impl IClientHandler for Logic {
    fn calculate_move(&mut self) -> socha::neutral::Move {
        info!("received mvoe");
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
    ComHandler::start("localhost:13050", None, &mut handler)?;
    Ok(())
}
