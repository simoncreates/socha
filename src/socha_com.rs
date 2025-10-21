#[cfg(feature = "unfinished")]
use crate::i_client_handler::IClientHandler;
use crate::incoming::{ReceivedComMessage, ReceivedRoom};
use crate::internal::{AdminMessage, ComMessage, Joined, Left, PreparedRoom, RoomMessage};
use crate::neutral::Direction;
use crate::outgoing::{make_join_prepared_xml, make_join_xml, make_move_xml};
use log::info;
use std::io::{self, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use strong_xml::XmlRead;

#[derive(Debug)]
pub enum ComMessageBuildErr {
    FailedBuildingMemento(String),
}

#[derive(Debug)]
pub enum ConnectionClosedErr {
    ServerClosed,
    ProtocolEnded,
    NoMessageReceivedFor(std::time::Duration),
}

#[derive(Debug)]
pub enum ReceiveErr {
    Io(io::Error),
    XmlError(strong_xml::XmlError),
    ConnectionClosed(ConnectionClosedErr),
    FailedToBuildRoomMessage(String),
    FailedToBuildAdminMessage(String),
}

impl From<io::Error> for ReceiveErr {
    fn from(err: io::Error) -> Self {
        ReceiveErr::Io(err)
    }
}
impl From<strong_xml::XmlError> for ReceiveErr {
    fn from(xml_err: strong_xml::XmlError) -> Self {
        Self::XmlError(xml_err)
    }
}

#[derive(Debug)]
pub enum SendErr {
    NoRoomId,
    FailedToBuildXml,
    Io(io::Error),
}
impl From<io::Error> for SendErr {
    fn from(err: io::Error) -> Self {
        SendErr::Io(err)
    }
}

/// communication error
#[derive(Debug)]
pub enum ComError {
    SendErr(SendErr),
    ReceiveErr(ReceiveErr),
}

impl From<SendErr> for ComError {
    fn from(value: SendErr) -> Self {
        ComError::SendErr(value)
    }
}

impl From<ReceiveErr> for ComError {
    fn from(value: ReceiveErr) -> Self {
        ComError::ReceiveErr(value)
    }
}

/// Connection helper for the Software-Challenge XML protocol
pub struct ComHandler {
    reader: BufReader<TcpStream>,
    buf: String,
    stream: TcpStream,
    pub room_id: Option<String>,
    msgs: Vec<ComMessage>,
    protocol_tag_found: bool,
}

impl ComHandler {
    #[cfg(feature = "unfinished")]
    pub fn start<H: IClientHandler>(
        addr: &str,
        opt_reservation_code: Option<&str>,
        handler: &mut H,
    ) -> Result<(), ComError> {
        let mut com = ComHandler::join(addr, opt_reservation_code)?;
        com.run_receive_loop(handler)?;
        Ok(())
    }

    /// BLOCKING: connect to `addr` and join a free room.
    pub fn join(addr: &str, opt_reservation_code: Option<&str>) -> Result<Self, ReceiveErr> {
        println!("connecting to {}", addr);
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;

        let join_xml = if let Some(res_code) = opt_reservation_code {
            make_join_prepared_xml(res_code).unwrap()
        } else {
            make_join_xml("swc_2026_piranhas", None).unwrap()
        };
        stream.write_all(b"<protocol>")?;
        stream.write_all(join_xml.as_bytes())?;
        stream.flush()?;

        let reader: BufReader<TcpStream> = BufReader::new(stream.try_clone()?);
        let mut com = ComHandler {
            reader,
            buf: String::new(),
            stream,
            room_id: None,
            msgs: Vec::new(),
            protocol_tag_found: false,
        };

        com.wait_for_and_rm_str("<protocol>")?;
        com.protocol_tag_found = true;
        Ok(com)
    }
    /// BLOCKING: connect to `addr`
    /// usually used for an admin client
    /// not recommeded when writing a bot-player
    pub fn connect_to_server(addr: &str) -> Result<Self, ReceiveErr> {
        println!("connecting to {}", addr);
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(false)?;

        stream.write_all(b"<protocol>")?;
        stream.flush()?;

        let reader: BufReader<TcpStream> = BufReader::new(stream.try_clone()?);

        Ok(ComHandler {
            reader,
            buf: String::new(),
            stream,
            room_id: None,
            msgs: Vec::new(),
            protocol_tag_found: false,
        })
    }
    #[cfg(feature = "unfinished")]
    fn run_receive_loop<H: IClientHandler>(&mut self, handler: &mut H) -> Result<(), ComError> {
        loop {
            let opt_msg = self.try_for_com_message()?;
            if let Some(msg) = opt_msg {
                match msg {
                    ComMessage::Joined(joined) => {
                        handler.on_game_joined(&joined.room_id);
                    }
                    ComMessage::Left(_left) => {
                        handler.on_game_left();
                    }
                    ComMessage::Admin(admin_msg) => {
                        let AdminMessage::Prepared(prep_room) = admin_msg;
                        handler.on_game_prepared(&prep_room);
                    }
                    ComMessage::Room(boxed_room_msg) => match *boxed_room_msg {
                        RoomMessage::Memento(state) => {
                            handler.on_gamestate_update(*state);
                        }
                        RoomMessage::MoveRequest => {
                            let cal_move = handler.calculate_move();
                            self.send_move(cal_move.from.0, cal_move.from.1, cal_move.dir)?;
                        }
                        RoomMessage::Result(res) => {
                            handler.on_game_result(&res);
                        }
                        RoomMessage::WelcomeMessage => {
                            handler.on_welcome_message();
                        }
                    },
                }
            }
        }
    }

    /// BLOCKING: wait until a `ComMessage` is available and return it.
    pub fn wait_for_com_message(&mut self, max_time: Duration) -> Result<ComMessage, ReceiveErr> {
        let res: Result<ComMessage, ReceiveErr>;
        let start_t = std::time::Instant::now();
        loop {
            let mut msgs = self.attempt_get_com_messages()?;
            self.msgs.append(&mut msgs);
            if !self.msgs.is_empty() {
                if cfg!(debug_assertions) {
                    info!("retrieving saved messages: {}", self.msgs.len());
                }
                res = Ok(self.msgs.remove(0));
                break;
            }
            if self.try_for_and_rm_str("<comMessage/>") {
                res = Err(ReceiveErr::ConnectionClosed(
                    ConnectionClosedErr::ProtocolEnded,
                ));
                break;
            }
            if start_t.elapsed() > max_time {
                return Err(ReceiveErr::ConnectionClosed(
                    ConnectionClosedErr::NoMessageReceivedFor(max_time),
                ));
            }
            self.try_read_new()?;
        }
        res
    }

    /// NONBLOCKING: try to read and return a `ComMessage` if available.
    pub fn try_for_com_message(&mut self) -> Result<Option<ComMessage>, ReceiveErr> {
        self.try_read_new()?;
        let mut msgs = self.attempt_get_com_messages()?;
        self.msgs.append(&mut msgs);
        if !self.msgs.is_empty() {
            if cfg!(debug_assertions) {
                info!("retrieving saved messages: {}", self.msgs.len());
            }
            return Ok(Some(self.msgs.remove(0)));
        }
        if self.try_for_and_rm_str("<comMessage/>") {
            return Err(ReceiveErr::ConnectionClosed(
                ConnectionClosedErr::ProtocolEnded,
            ));
        }
        Ok(None)
    }

    /// BLOCKING: wait until `str` appears in the buffer, then remove it
    fn wait_for_and_rm_str(&mut self, str: &str) -> Result<(), ReceiveErr> {
        loop {
            if let Some(pos) = self.buf.find(str) {
                self.buf.drain(..pos + str.len());
                break;
            }
            self.try_read_new()?;
        }
        Ok(())
    }

    /// NONBLOCKING: remove `str` if present and return whether it was found.
    fn try_for_and_rm_str(&mut self, str: &str) -> bool {
        if let Some(pos) = self.buf.find(str) {
            self.buf.drain(..pos + str.len());
            return true;
        }
        false
    }

    /// NONBLOCKING: try to parse a `<comMessage>...</comMessage>` from buffer.
    fn get_com_msg_and_rm(&mut self) -> Option<ReceivedComMessage> {
        // removing the protocol tag to avoid parsing issues
        self.check_for_protocol_tag();

        let prepared_buf = format!("<comMessage>{}</comMessage>", self.buf);
        let rs_msg = ReceivedComMessage::from_str(&prepared_buf);

        let msg = match rs_msg {
            Ok(m) => m,
            Err(e) => {
                if cfg!(debug_assertions) {
                    info!("");
                    info!("xml parse error: \n \"{:?}\"", e);
                    info!("current buffer: \n {}", prepared_buf);
                    info!("");
                }
                return None;
            }
        };
        self.buf.clear();
        Some(msg)
    }

    #[inline]
    /// looks for the protocol tag and removes it if found
    /// gets skipped if one was found alr
    fn check_for_protocol_tag(&mut self) {
        if !self.protocol_tag_found && self.try_for_and_rm_str("<protocol>") {
            self.protocol_tag_found = true;
        };
    }

    /// NONBLOCKING: try to read available bytes and append to internal buffer.
    fn try_read_new(&mut self) -> Result<(), ReceiveErr> {
        let mut tmp = [0_u8; 4096];

        match self.reader.read(&mut tmp) {
            Ok(n) => {
                if n == 0 {
                    return Err(ReceiveErr::ConnectionClosed(
                        ConnectionClosedErr::ServerClosed,
                    ));
                }
                let chunk = String::from_utf8_lossy(&tmp[..n]);
                self.buf.push_str(&chunk);

                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(()),
            Err(e) => Err(ReceiveErr::Io(e)),
        }
    }

    fn create_com_message_from_received_room(
        &self,
        received_room: ReceivedRoom,
    ) -> Result<ComMessage, ComMessageBuildErr> {
        let rm_msg = RoomMessage::try_from(received_room)
            .map_err(ComMessageBuildErr::FailedBuildingMemento)?;

        Ok(ComMessage::Room(Box::new(rm_msg)))
    }

    fn attempt_get_com_messages(&mut self) -> Result<Vec<ComMessage>, ReceiveErr> {
        let mut messages = Vec::new();
        if let Some(recv_com_msg) = self.get_com_msg_and_rm() {
            //todo: make code better (.___.)
            if let Some(recv_joined) = &recv_com_msg.joined {
                if let Some(room_id) = &recv_joined.room_id {
                    self.room_id = Some(room_id.clone());
                    messages.push(ComMessage::Joined(Joined {
                        room_id: room_id.clone(),
                    }));
                }
            }

            if let Some(recv_left) = &recv_com_msg.left {
                if let Some(room_id) = &recv_left.room_id {
                    self.room_id = Some(room_id.clone());
                    messages.push(ComMessage::Left(Left {
                        room_id: room_id.clone(),
                    }));
                }
            }
            if let Some(recv_admin_prepared) = &recv_com_msg.admin_prepared {
                let reservations = if recv_admin_prepared.admin_reservation.len() == 2 {
                    (
                        recv_admin_prepared.admin_reservation[0]
                            .reservation_id
                            .clone(),
                        recv_admin_prepared.admin_reservation[1]
                            .reservation_id
                            .clone(),
                    )
                } else {
                    return Err(ReceiveErr::FailedToBuildAdminMessage(
                        "a received reservation should hold two reserved spots".to_string(),
                    ));
                };
                messages.push(ComMessage::Admin(AdminMessage::Prepared(PreparedRoom {
                    reservations,
                    room_id: recv_admin_prepared.room_id.clone(),
                })));
            }

            for room in recv_com_msg.room {
                if let Ok(msg) = self.create_com_message_from_received_room(room) {
                    messages.push(msg);
                }
            }
        }
        Ok(messages)
    }

    /// Send a move. Returns `NoRoomId` if not joined yet.
    pub fn send_move(&mut self, x: u8, y: u8, dir: Direction) -> Result<(), SendErr> {
        if let Some(room) = &self.room_id {
            let xml = make_move_xml(room, x.into(), y.into(), dir)
                .map_err(|_| SendErr::FailedToBuildXml)?;

            self.stream.write_all(xml.as_bytes())?;
            self.stream.flush()?;
            Ok(())
        } else {
            Err(SendErr::NoRoomId)
        }
    }

    fn send_raw(&mut self, xml: &str) -> Result<(), SendErr> {
        self.stream.write_all(xml.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    //___ admin ___

    /// authenticate as admin
    pub fn send_admin_authenticate(&mut self, password: &str) -> Result<(), SendErr> {
        let xml = crate::outgoing::make_authenticate_xml(password)
            .map_err(|_| SendErr::FailedToBuildXml)?;
        self.send_raw(&xml)
    }

    /// observe a room
    pub fn send_admin_observe(&mut self, room_id: &str) -> Result<(), SendErr> {
        let xml =
            crate::outgoing::make_observe_xml(room_id).map_err(|_| SendErr::FailedToBuildXml)?;
        self.send_raw(&xml)
    }

    /// pause / resume a room
    pub fn send_admin_pause(&mut self, room_id: &str, pause: bool) -> Result<(), SendErr> {
        let xml = crate::outgoing::make_pause_xml(room_id, pause)
            .map_err(|_| SendErr::FailedToBuildXml)?;
        self.send_raw(&xml)
    }

    /// step a paused room once
    pub fn send_admin_step(&mut self, room_id: &str) -> Result<(), SendErr> {
        let xml = crate::outgoing::make_step_xml(room_id).map_err(|_| SendErr::FailedToBuildXml)?;
        self.send_raw(&xml)
    }

    /// cancel a room
    pub fn send_admin_cancel(&mut self, room_id: &str) -> Result<(), SendErr> {
        let xml =
            crate::outgoing::make_cancel_xml(room_id).map_err(|_| SendErr::FailedToBuildXml)?;
        self.send_raw(&xml)
    }

    /// prepare a new room
    /// slots is a slice of tuples: (display_name, can_timeout, reserved)
    pub fn send_admin_prepare(
        &mut self,
        pause: bool,
        slots: &[PrepareSlot],
    ) -> Result<(), SendErr> {
        let xml = crate::outgoing::make_prepare_xml("swc_2026_piranhas", pause, slots)
            .map_err(|_| SendErr::FailedToBuildXml)?;
        self.send_raw(&xml)
    }
}

#[derive(Debug, Clone)]
pub struct PrepareSlot {
    pub displayname: String,
    pub can_timeout: bool,
    pub reserved: bool,
}

impl PrepareSlot {
    pub fn new(displayname: String, can_timeout: bool, reserved: bool) -> Self {
        PrepareSlot {
            displayname,
            can_timeout,
            reserved,
        }
    }
}
