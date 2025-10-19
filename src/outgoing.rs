use std::error::Error;
use strong_xml::{XmlRead, XmlWrite};

use crate::neutral::Direction;

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "join")]
pub struct Join {
    #[xml(attr = "gameType")]
    pub game_type: String,
    #[xml(attr = "participantId")]
    pub participant_id: Option<String>,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "joinPrepared")]
pub struct JoinPrepared {
    #[xml(attr = "reservationCode")]
    pub reservation_code: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "from")]
pub struct FromPos {
    #[xml(attr = "x")]
    pub x: u32,
    #[xml(attr = "y")]
    pub y: u32,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "direction")]
pub struct OutgoingDirection {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "data")]
pub struct DataMove {
    #[xml(attr = "class")]
    pub class: String,

    #[xml(child = "from")]
    pub from: FromPos,

    #[xml(child = "direction")]
    pub direction: OutgoingDirection,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "room")]
pub struct OutgoingRoom {
    #[xml(attr = "roomId")]
    pub room_id: String,

    #[xml(child = "data")]
    pub data: DataMove,
}

pub fn make_join_xml(
    game_type: &str,
    participant_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let join = Join {
        game_type: game_type.to_string(),
        participant_id: participant_id.map(|s| s.to_string()),
    };
    Ok(join.to_string()?)
}

pub fn make_join_prepared_xml(reservation_code: &str) -> Result<String, Box<dyn Error>> {
    let join_prepared = JoinPrepared {
        reservation_code: reservation_code.to_string(),
    };
    Ok(join_prepared.to_string()?)
}

pub fn make_move_xml(
    room_id: &str,
    x: u32,
    y: u32,
    direction: Direction,
) -> Result<String, Box<dyn Error>> {
    let data = DataMove {
        class: "move".to_string(),
        from: FromPos { x, y },
        direction: OutgoingDirection {
            value: direction.to_string(),
        },
    };

    let room = OutgoingRoom {
        room_id: room_id.to_string(),
        data,
    };

    Ok(room.to_string()?)
}

// ____ auth ____

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "authenticate")]
pub struct Authenticate {
    #[xml(attr = "password")]
    pub password: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "observe")]
pub struct Observe {
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "pause")]
pub struct Pause {
    #[xml(attr = "roomId")]
    pub room_id: String,
    // use string "true"/"false" to exactly match docs
    #[xml(attr = "pause")]
    pub pause: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "step")]
pub struct Step {
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "cancel")]
pub struct Cancel {
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "slot")]
pub struct Slot {
    #[xml(attr = "displayName")]
    pub display_name: String,
    // "true"/"false" strings to match doc examples
    #[xml(attr = "canTimeout")]
    pub can_timeout: String,
    #[xml(attr = "reserved")]
    pub reserved: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "prepare")]
pub struct Prepare {
    #[xml(attr = "gameType")]
    pub game_type: String,
    #[xml(attr = "pause")]
    pub pause: String,
    #[xml(child = "slot")]
    pub slots: Vec<Slot>,
}

/// helpers

pub fn make_authenticate_xml(password: &str) -> Result<String, Box<dyn Error>> {
    let auth = Authenticate {
        password: password.to_string(),
    };
    Ok(auth.to_string()?)
}

pub fn make_observe_xml(room_id: &str) -> Result<String, Box<dyn Error>> {
    let o = Observe {
        room_id: room_id.to_string(),
    };
    Ok(o.to_string()?)
}

pub fn make_pause_xml(room_id: &str, pause: bool) -> Result<String, Box<dyn Error>> {
    let p = Pause {
        room_id: room_id.to_string(),
        pause: if pause { "true".into() } else { "false".into() },
    };
    Ok(p.to_string()?)
}

pub fn make_step_xml(room_id: &str) -> Result<String, Box<dyn Error>> {
    let s = Step {
        room_id: room_id.to_string(),
    };
    Ok(s.to_string()?)
}

pub fn make_cancel_xml(room_id: &str) -> Result<String, Box<dyn Error>> {
    let c = Cancel {
        room_id: room_id.to_string(),
    };
    Ok(c.to_string()?)
}

/// `slots` is a slice of tuples: (display_name, can_timeout, reserved)
pub fn make_prepare_xml(
    game_type: &str,
    pause: bool,
    slots: &[(&str, bool, bool)],
) -> Result<String, Box<dyn Error>> {
    let slots_vec = slots
        .iter()
        .map(|(dn, ct, res)| Slot {
            display_name: dn.to_string(),
            can_timeout: if *ct { "true".into() } else { "false".into() },
            reserved: if *res { "true".into() } else { "false".into() },
        })
        .collect();
    let p = Prepare {
        game_type: game_type.to_string(),
        pause: if pause { "true".into() } else { "false".into() },
        slots: slots_vec,
    };
    Ok(p.to_string()?)
}
