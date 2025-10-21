#![allow(clippy::needless_late_init, unused_must_use)]
// allowing needless_late_init due to warnings coming from macros inside the StrongXml crate
use strong_xml::{XmlRead, XmlWrite};

#[derive(Debug, XmlRead, XmlWrite, Clone)]
#[xml(tag = "field")]
pub struct Field {
    #[xml(text)]
    pub raw: String,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "row")]
pub struct ReicevedRow {
    #[xml(child = "field")]
    pub fields: Vec<Field>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "board")]
pub struct ReceivedBoard {
    #[xml(child = "row")]
    pub rows: Vec<ReicevedRow>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "from")]
pub struct ReicevedFromPos {
    #[xml(attr = "x")]
    pub x: u8,
    #[xml(attr = "y")]
    pub y: u8,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "direction")]
pub struct ReceivedDirection {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "lastMove")]
pub struct ReceivedLastMove {
    #[xml(child = "from")]
    pub from: Option<ReicevedFromPos>,

    #[xml(child = "direction")]
    pub direction: Option<ReceivedDirection>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "state")]
pub struct ReceivedState {
    #[xml(attr = "class")]
    pub class: Option<String>,

    #[xml(attr = "startTeam")]
    pub start_team: Option<String>,

    #[xml(attr = "turn")]
    pub turn: Option<u32>,

    #[xml(child = "lastMove")]
    pub last_move: Option<ReceivedLastMove>,

    #[xml(child = "board")]
    pub board: Option<ReceivedBoard>,
}
#[derive(Debug, XmlRead, XmlWrite, PartialEq, Eq)]
#[xml(tag = "aggregation")]
pub struct ReceivedAggregation {
    #[xml(text)]
    pub agr_content: String,
}

#[derive(Debug, XmlRead, XmlWrite, PartialEq, Eq)]
#[xml(tag = "relevantForRanking")]
pub struct ReceivedRelevantForRanking {
    #[xml(text)]
    pub rfr_content: String,
}

#[derive(Debug, XmlRead, XmlWrite, PartialEq, Eq)]
#[xml(tag = "fragment")]
pub struct ReceivedFragment {
    #[xml(attr = "name")]
    pub frag_name: Option<String>,
    #[xml(child = "aggregation")]
    pub aggregation: Option<ReceivedAggregation>,
    #[xml(child = "relevantForRanking")]
    pub relevant_for_ranking: Option<ReceivedRelevantForRanking>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "player")]
pub struct ReceivedPlayer {
    #[xml(attr = "team")]
    pub team: Option<String>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "part")]
pub struct ReceivedPartScore {
    #[xml(text)]
    pub part_content: String,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "score")]
pub struct ReceivedScore {
    #[xml(child = "part")]
    pub parts: Vec<ReceivedPartScore>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "entry")]
pub struct ReceivedEntry {
    #[xml(child = "player")]
    pub player: ReceivedPlayer,
    #[xml(child = "score")]
    pub score: Option<ReceivedScore>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "definition")]
pub struct ReceivedDefinition {
    #[xml(child = "fragment")]
    pub fragments: Vec<ReceivedFragment>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "scores")]
pub struct ReceivedScores {
    #[xml(child = "entry")]
    pub entries: Vec<ReceivedEntry>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "winner")]
pub struct ReceivedWinner {
    #[xml(attr = "team")]
    pub team: Option<String>,
    #[xml(attr = "regular")]
    pub regular: Option<String>,
    #[xml(attr = "reason")]
    pub reason: Option<String>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "data")]
pub struct ReceivedData {
    #[xml(attr = "class")]
    pub class: Option<String>,

    #[xml(child = "definition")]
    pub definition: Option<ReceivedDefinition>,

    #[xml(child = "scores")]
    pub scores: Option<ReceivedScores>,

    #[xml(child = "winner")]
    pub winner: Option<ReceivedWinner>,

    #[xml(child = "state")]
    pub state: Option<ReceivedState>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "room")]
pub struct ReceivedRoom {
    #[xml(attr = "roomId")]
    pub room_id: Option<String>,

    #[xml(child = "data")]
    pub data: Option<ReceivedData>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "joined")]
pub struct ReceivedJoined {
    #[xml(attr = "roomId")]
    pub room_id: Option<String>,
}
#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "left")]
pub struct ReceivedLeft {
    #[xml(attr = "roomId")]
    pub room_id: Option<String>,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "comMessage")]
pub struct ReceivedComMessage {
    #[xml(child = "left")]
    pub left: Option<ReceivedLeft>,
    #[xml(child = "joined")]
    pub joined: Option<ReceivedJoined>,
    #[xml(child = "room")]
    pub room: Vec<ReceivedRoom>,
    #[xml(child = "prepared")]
    pub admin_prepared: Option<ReceivedAdminPrepared>,
}

// ___ admin ____

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "prepared")]
pub struct ReceivedAdminPrepared {
    #[xml(child = "reservation")]
    pub admin_reservation: Vec<ReceivedAdminReservation>,
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlRead, XmlWrite)]
#[xml(tag = "reservation")]
pub struct ReceivedAdminReservation {
    #[xml(text)]
    pub reservation_id: String,
}
