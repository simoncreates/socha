use std::io;
use std::time::Duration;

use log::LevelFilter;
use simple_logging::log_to_file;
use socha::internal::{ComMessage, RoomMessage};
use socha::socha_com::{ReceiveErr, SochaCom};

// todo: finish
fn main() -> io::Result<()> {
    log_to_file("admin_com.log", LevelFilter::Info).unwrap();

    let mut com = match SochaCom::connect_to_server("localhost:13050") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("connect failed: {:?}", e);
            return Err(std::io::Error::other("connect failed"));
        }
    };

    // auth
    if let Err(e) = com.send_admin_authenticate("examplepassword") {
        eprintln!("admin authenticate failed: {:?}", e);
        return Err(std::io::Error::other("admin auth failed"));
    }
    eprintln!("sent admin authenticate");

    let slots = [("player1", true, true), ("player2", true, true)];
    if let Err(e) = com.send_admin_prepare(true, &slots) {
        eprintln!("prepare failed: {:?}", e);
        return Err(std::io::Error::other("prepare failed"));
    }
    eprintln!("sent prepare (paused)");

    let mut sent_observe = false;

    loop {
        match com.wait_for_com_message(Duration::from_secs(50)) {
            Ok(com_message) => {
                if !sent_observe {
                    if let Some(room_id) = com.room_id.clone() {
                        eprintln!("found room_id '{}', sending observe", room_id);
                        if let Err(e) = com.send_admin_observe(&room_id) {
                            eprintln!("send_observe failed: {:?}", e);
                            return Err(std::io::Error::other("observe failed"));
                        }
                    }
                    sent_observe = true;
                }

                match com_message {
                    ComMessage::Joined(joined) => {
                        eprintln!("(incoming) joined room {}", joined.room_id);
                    }
                    ComMessage::Left(left) => {
                        eprintln!("(incoming) left room {}", left.room_id);
                    }
                    ComMessage::Room(room_msg) => match *room_msg {
                        RoomMessage::Memento(state) => {
                            eprintln!("(incoming) got board: \n{}", state.board);
                            eprintln!("turn {}, class {:?}", state.turn, state.class);
                        }
                        RoomMessage::WelcomeMessage => {
                            eprintln!("(incoming) got welcome message");
                        }
                        RoomMessage::MoveRequest => {
                            eprintln!("(incoming) got move request");
                        }
                        RoomMessage::Result(result) => {
                            eprintln!("(incoming) got result: \n{:#?}", result);
                        }
                    },
                    ComMessage::Admin(admin) => {
                        eprintln!("admin message: {:?}", admin);
                    }
                }
            }

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
