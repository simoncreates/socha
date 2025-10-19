use std::{fmt, str::FromStr};

use crate::{
    incoming::{ReceivedBoard, ReceivedData, ReceivedRoom, ReceivedState},
    neutral::{Move, PiranhaField, Team},
};

#[derive(Debug)]
pub struct Joined {
    pub room_id: String,
}

#[derive(Debug)]
/// opposite of joined
pub struct Left {
    pub room_id: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Row {
    /// from left to right
    pub fields: [PiranhaField; 10],
}

#[derive(Debug)]
pub struct Board {
    /// from bottom to top
    pub rows: [Row; 10],
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows.iter().rev() {
            for field in row.fields.iter() {
                write!(f, "{:<12}", field)?;
                write!(f, " ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl TryFrom<ReceivedBoard> for Board {
    type Error = String;
    fn try_from(recv_board: ReceivedBoard) -> Result<Self, Self::Error> {
        if recv_board.rows.len() != 10 {
            return Err(format!(
                "board should contain exactly 10 rows, but has {}",
                recv_board.rows.len()
            ));
        }
        let mut rows: [Row; 10] = Default::default();
        for (i, recv_row) in recv_board.rows.into_iter().enumerate() {
            if recv_row.fields.len() != 10 {
                return Err(format!(
                    "row {} should contain exactly 10 fields, but has {}",
                    i,
                    recv_row.fields.len()
                ));
            }
            let mut fields: [PiranhaField; 10] = Default::default();
            for (j, recv_field) in recv_row.fields.into_iter().enumerate() {
                let field = PiranhaField::from_str(recv_field.raw.as_ref())?;
                fields[j] = field;
            }
            rows[i] = Row { fields };
        }
        Ok(Board { rows })
    }
}

#[derive(Debug)]
pub struct GameState {
    // todo handler class
    pub class: Option<String>,
    pub start_team: Team,
    pub turn: u32,
    pub board: Board,
    pub last_move: Option<Move>,
}

impl TryFrom<ReceivedState> for GameState {
    type Error = String;
    fn try_from(recv_state: ReceivedState) -> Result<Self, Self::Error> {
        let start_team = if let Some(start_team_str) = recv_state.start_team {
            Team::try_from(start_team_str.as_ref())?
        } else {
            return Err(
                "ReceivedState should contain a start team when converting to GameState"
                    .to_string(),
            );
        };

        let turn = if let Some(t) = recv_state.turn {
            t
        } else {
            return Err(
                "ReceivedState should contain \"turn\" when converting to GameState".to_string(),
            );
        };

        let last_move = if let Some(recv_last_move) = recv_state.last_move {
            Some(Move::try_from(&recv_last_move)?)
        } else {
            None
        };

        let board = if let Some(recv_board) = recv_state.board {
            Board::try_from(recv_board)?
        } else {
            return Err(
                "ReceivedState should contain \"board\" when converting to GameState".to_string(),
            );
        };

        Ok(GameState {
            // todo: stop pushing class directly
            class: recv_state.class,
            start_team,
            turn,
            board,
            last_move,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScoreTypes {
    Siegpunkte,
    Schwarmgröße,
}

#[derive(Debug, Clone, Copy)]
pub enum AggregationTypes {
    Sum,
    Average,
}

#[derive(Debug, Clone, Copy)]
pub struct Scores {
    pub score_type: ScoreTypes,
    pub value: u32,
    pub aggregation_type: AggregationTypes,
    pub relevant_for_ranking: bool,
}

#[derive(Debug, Clone)]
pub struct Winner {
    pub team: Team,
    pub regular: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GameResult {
    pub player1_result: Vec<Scores>,
    pub player2_result: Vec<Scores>,
    pub winner: Option<Winner>,
}

impl TryFrom<ReceivedData> for GameResult {
    type Error = String;
    fn try_from(recv_data: ReceivedData) -> Result<Self, Self::Error> {
        let winner = if let Some(scores) = recv_data.winner {
            let team = if let Some(team) = scores.team.as_ref() {
                Team::try_from(team.as_ref())?
            } else {
                return Err("missing winner team".to_string());
            };
            let regular = if let Some(regular) = scores.regular.as_ref() {
                regular.parse::<bool>().map_err(|e| e.to_string())?
            } else {
                return Err("missing winner regular".to_string());
            };
            let reason = scores.reason;
            Some(Winner {
                team,
                regular,
                reason,
            })
        } else {
            None
        };
        let mut definitions = Vec::new();
        if let Some(def) = &recv_data.definition {
            for frag in &def.fragments {
                let score_type = if let Some(name) = frag.frag_name.as_ref() {
                    match name.as_ref() {
                        "Siegpunkte" => ScoreTypes::Siegpunkte,
                        // todo: check if it is written as "Schwarmgr..e"
                        "Schwarmgröße" => ScoreTypes::Schwarmgröße,
                        other => {
                            return Err(format!("unknown score type '{}'", other));
                        }
                    }
                } else {
                    return Err("missing fragment name".to_string());
                };
                let aggregation_type = if let Some(agr) = frag.aggregation.as_ref() {
                    match agr.agr_content.as_ref() {
                        "SUM" => AggregationTypes::Sum,
                        "AVERAGE" => AggregationTypes::Average,
                        other => {
                            return Err(format!("unknown aggregation type '{}'", other));
                        }
                    }
                } else {
                    return Err("missing aggregation type".to_string());
                };
                let relevant_for_ranking = if let Some(rfr) = frag.relevant_for_ranking.as_ref() {
                    rfr.rfr_content.parse::<bool>().map_err(|e| e.to_string())?
                } else {
                    return Err("missing relevantForRanking".to_string());
                };
                definitions.push((score_type, aggregation_type, relevant_for_ranking));
            }
        }

        let mut player1_result = Vec::new();
        let mut player2_result = Vec::new();
        for entry in recv_data.scores.ok_or("missing scores")?.entries {
            let team = if let Some(player) = entry.player.team.as_ref() {
                Team::try_from(player.as_ref())?
            } else {
                return Err("missing player team".to_string());
            };
            let score = if let Some(score) = entry.score {
                score
            } else {
                return Err("missing player score".to_string());
            };
            if score.parts.len() != definitions.len() {
                return Err(format!(
                    "score parts length {} does not match definitions length {}",
                    score.parts.len(),
                    definitions.len()
                ));
            }
            let mut scores = Vec::new();
            for (part, def) in score.parts.into_iter().zip(definitions.iter()) {
                let value = part
                    .part_content
                    .parse::<u32>()
                    .map_err(|e| e.to_string())?;
                let score = Scores {
                    score_type: def.0,
                    value,
                    aggregation_type: def.1,
                    relevant_for_ranking: def.2,
                };
                scores.push(score);
            }
            match team {
                Team::One => player1_result = scores,
                Team::Two => player2_result = scores,
            }
        }

        Ok(GameResult {
            player1_result,
            player2_result,
            winner,
        })
    }
}

#[derive(Debug)]
pub enum RoomMessage {
    Memento(Box<GameState>),
    Result(Box<GameResult>),
    WelcomeMessage,
    MoveRequest,
}
#[derive(Debug)]
pub struct PreparedRoom {
    pub reservations: (String, String),
    pub room_id: String,
}
#[derive(Debug)]
pub enum AdminMessage {
    /// (reservation, reservation)
    Prepared(PreparedRoom),
}

impl TryFrom<ReceivedRoom> for RoomMessage {
    type Error = String;
    fn try_from(recv_room: ReceivedRoom) -> Result<Self, Self::Error> {
        if let Some(data) = recv_room.data {
            if let Some(class) = &data.class {
                match class.as_str() {
                    "memento" => {
                        if let Some(state) = data.state {
                            if state.class.as_ref() != Some(&"state".to_string()) {
                                return Err("Received data with a class of memento should contain the a <state> element".to_string());
                            }
                            let state = GameState::try_from(state)?;
                            Ok(RoomMessage::Memento(Box::new(state)))
                        } else {
                            Err("Received data with a class of memento should contain the ReceivedState".to_string())
                        }
                    }
                    "result" => {
                        if let Ok(result) = GameResult::try_from(data) {
                            Ok(RoomMessage::Result(Box::new(result)))
                        } else {
                            Err("Received data with a class of result should contain the ReceivedResult".to_string())
                        }
                    }
                    "welcomeMessage" => Ok(RoomMessage::WelcomeMessage),
                    "moveRequest" => Ok(RoomMessage::MoveRequest),
                    other => Err(format!("unknown room message class '{}'", other)),
                }
            } else {
                Err("missing room message class".to_string())
            }
        } else {
            Err("missing room message data".to_string())
        }
    }
}

#[derive(Debug)]
pub enum ComMessage {
    Joined(Joined),
    Left(Left),
    Room(Box<RoomMessage>),
    Admin(AdminMessage),
}
