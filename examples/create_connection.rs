use std::io;
use std::process::exit;
use std::time::Duration;

use log::LevelFilter;
use simple_logging::log_to_file;
use socha::error::ReceiveErr;
use socha::internal::{ComMessage, RoomMessage};
use socha::socha_com::ComHandler;

fn main() -> io::Result<()> {
    log_to_file("com.log", LevelFilter::Info).unwrap();
    let mut com = match ComHandler::join("localhost:13050", None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("connect/join failed: {:?}", e);
            return Err(std::io::Error::other("join failed"));
        }
    };

    loop {
        match com.wait_for_com_message(Duration::from_secs(50)) {
            Ok(com_message) => match com_message {
                ComMessage::Joined(joined) => {
                    eprintln!("joined room {}", joined.room_id);
                }
                ComMessage::Left(left) => {
                    eprintln!("left room {}", left.room_id);
                    // if you want to keep your client running, in this example, i will stop the bot to avoid idling
                    exit(0)
                }
                ComMessage::Room(room_msg) => match *room_msg {
                    RoomMessage::Memento(state) => {
                        eprintln!("got board: \n{}", state.board);
                        eprintln!("turn {}, class {:?}", state.turn, state.class);
                    }
                    RoomMessage::WelcomeMessage => {
                        eprintln!("got welcome message");
                    }
                    RoomMessage::MoveRequest => {
                        eprintln!("got move request");
                    }
                    RoomMessage::Result(result) => {
                        eprintln!("got result: \n{:#?}", result);
                    }
                },
                ComMessage::Admin(_) => {}
            },
            Err(ReceiveErr::XmlError(e)) => {
                eprintln!("XML parse error: {:?}", e);
                continue;
            }
            Err(e) => {
                eprintln!("fatal receive error: {:#?}", e);
                break;
            }
        }
    }
    Ok(())
}
