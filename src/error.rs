use std::io;

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
