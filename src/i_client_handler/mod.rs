#[cfg(feature = "unfinished")]
use std::time::Duration;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use log::info;
pub mod handler_trait;

#[cfg(feature = "unfinished")]
use crate::{error::ReceiveErr, i_client_handler::handler_trait::IClientHandler};
use crate::{
    internal::{ComMessage, RoomMessage},
    neutral::Move,
};

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

#[cfg(feature = "unfinished")]
pub fn start_iclient<I>(
    addr: &str,
    opt_reservation_code: Option<&str>,
    i_client_handler: &mut I,
    thread_sleep_time: Duration,
    timeout: Duration,
) -> Result<(), ReceiveErr>
where
    I: IClientHandler,
{
    use crate::socha_com::ComHandler;

    let mut com = ComHandler::join(addr, opt_reservation_code)?;
    let (msg_tx, msg_rx) = unbounded::<ComMessage>();
    let (out_tx, out_rx) = unbounded::<SendCommnad>();

    let worker_rx = msg_rx.clone();

    let reader_handle = std::thread::spawn(move || loop {
        // messages from server
        match com.try_for_com_message() {
            Ok(Some(msg)) => {
                println!("received:{:?}", msg);

                if msg_tx.send(msg).is_err() {
                    break;
                }
            }
            Ok(None) => {
                std::thread::sleep(thread_sleep_time);
            }
            Err(_e) => {
                break;
            }
        }
        // forwarding messages from the main loop to the server
        if let Ok(out_msg) = out_rx.try_recv() {
            match out_msg {
                SendCommnad::Move(mv) => {
                    let _ = com.send_move(mv.from.0, mv.from.1, mv.dir);
                }
                SendCommnad::SendRaw { xml } => {
                    let _ = com.send_raw(&xml);
                }
                SendCommnad::Admin(_admin_cmd) => {
                    // todo: implement (shouldnt be necessary)
                }
            }
        }
    });

    loop {
        use crossbeam_channel::TryRecvError;

        use crate::internal::RoomMessage;

        match worker_rx.try_recv() {
            Ok(com_message) => match com_message {
                ComMessage::Joined(joined) => {
                    info!("joined room {}", joined.room_id);
                    i_client_handler.on_game_joined(&joined.room_id);
                }
                ComMessage::Left(left) => {
                    info!("left room {}", left.room_id);
                    i_client_handler.on_game_left();
                    break;
                }
                ComMessage::Room(room_msg) => match *room_msg {
                    RoomMessage::Memento(state) => {
                        info!("got board: \n{}", state.board);
                        info!("turn {}, class {:?}", state.turn, state.class);
                        i_client_handler.on_gamestate_update(*state);
                    }
                    RoomMessage::WelcomeMessage => {
                        info!("got welcome message");
                        i_client_handler.on_welcome_message();
                    }
                    RoomMessage::MoveRequest => {
                        info!("got move request");
                        let cancel_handler = ComCancelHandler::new_from_receiver(
                            worker_rx.clone(),
                            timeout,
                            thread_sleep_time,
                        );
                        let mv = i_client_handler.calculate_move(cancel_handler);
                        out_tx.send(SendCommnad::Move(mv)).unwrap();
                    }
                    RoomMessage::Result(result) => {
                        info!("got result: \n{:#?}", result);
                        i_client_handler.on_game_result(&result);
                    }
                },
                ComMessage::Admin(_) => {}
            },
            Err(TryRecvError::Empty) => {
                std::thread::sleep(thread_sleep_time);
            }
            Err(TryRecvError::Disconnected) => {
                info!("worker channel disconnected, exiting");
                break;
            }
        }
    }
    let _ = reader_handle.join();
    Ok(())
}

use crossbeam_channel::{unbounded, Receiver};

pub struct ComCancelHandler {
    flag: Arc<AtomicBool>,
    watchdog_handle: Option<JoinHandle<()>>,
}

impl ComCancelHandler {
    pub fn new_from_receiver(
        rx: Receiver<ComMessage>,
        timeout: Duration,
        thread_sleep_time: Duration,
    ) -> Self {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let handle = std::thread::spawn(move || {
            // drain messages
            while let Ok(_m) = rx.try_recv() {}

            let start = Instant::now();
            loop {
                if flag_clone.load(Ordering::SeqCst) {
                    break;
                }
                match rx.try_recv() {
                    Ok(msg) => {
                        match msg {
                            ComMessage::Room(room_msg) => {
                                if *room_msg == RoomMessage::MoveRequest {
                                    // next move request arrived, cancel
                                    flag_clone.store(true, Ordering::SeqCst);
                                    break;
                                }
                            }
                            ComMessage::Left(_id) => {
                                // left the room, cancel
                                flag_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(crossbeam_channel::TryRecvError::Empty) => {}
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        flag_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }

                if start.elapsed() >= timeout {
                    flag_clone.store(true, Ordering::SeqCst);
                    break;
                }

                thread::sleep(thread_sleep_time);
            }
        });

        ComCancelHandler {
            flag,
            watchdog_handle: Some(handle),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

impl Drop for ComCancelHandler {
    fn drop(&mut self) {
        if let Some(h) = self.watchdog_handle.take() {
            let _ = h.join();
        }
    }
}
