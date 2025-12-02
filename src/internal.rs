use std::{fmt, str::FromStr};

use crate::{
    incoming::{ReceivedBoard, ReceivedData, ReceivedRoom, ReceivedState},
    neutral::{Direction, Move, PiranhaField, Team},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Joined {
    pub room_id: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// opposite of joined
pub struct Left {
    pub room_id: String,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Row {
    /// from left to right
    pub fields: [PiranhaField; 10],
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
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

impl Board {
    pub fn in_bounds(x: i32, y: i32) -> bool {
        (0..10).contains(&x) && (0..10).contains(&y)
    }

    pub fn get(&self, x: usize, y: usize) -> &PiranhaField {
        &self.rows[y].fields[x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut PiranhaField {
        &mut self.rows[y].fields[x]
    }

    pub fn count_fishes_on_axis(&self, x: usize, y: usize, dir: Direction) -> u8 {
        let (dx, dy) = dir.to_delta();

        let mut cnt: u8 = 0;

        if matches!(self.get(x, y), PiranhaField::Fish { .. }) {
            cnt += 1;
        }
        let mut cx = x as i32 + dx;
        let mut cy = y as i32 + dy;

        while Board::in_bounds(cx, cy) {
            if let PiranhaField::Fish { .. } = self.get(cx as usize, cy as usize) {
                cnt += 1;
            }
            cx += dx;
            cy += dy;
        }

        let (bdx, bdy) = (-dx, -dy);
        let mut cx = x as i32 + bdx;
        let mut cy = y as i32 + bdy;

        while Board::in_bounds(cx, cy) {
            if let PiranhaField::Fish { .. } = self.get(cx as usize, cy as usize) {
                cnt += 1;
            }
            cx += bdx;
            cy += bdy;
        }

        cnt
    }

    pub fn check_allowed(
        board: &Self,
        x: usize,
        y: usize,
        dir: Direction,
        dis: u8,
        us_team: Team,
    ) -> bool {
        if dis == 0 {
            return false;
        }

        let opp_team = match us_team {
            Team::One => Team::Two,
            Team::Two => Team::One,
        };

        let mut cx = x as i32;
        let mut cy = y as i32;
        let (dx, dy) = dir.to_delta();
        let steps = dis as i32;

        for _step in 1..steps {
            cx += dx;
            cy += dy;

            if !Board::in_bounds(cx, cy) {
                return false;
            }

            let p_field = Board::get(board, cx as usize, cy as usize);

            // opp fish in the way
            if matches!(p_field, PiranhaField::Fish { team, .. } if *team == opp_team) {
                return false;
            }
        }

        // check goal field
        cx += dx;
        cy += dy;

        if !Board::in_bounds(cx, cy) {
            return false;
        }

        let goal_field = Board::get(board, cx as usize, cy as usize);

        match goal_field {
            PiranhaField::Squid => false,
            PiranhaField::Fish { team, .. } if *team == us_team => false,
            PiranhaField::Empty | PiranhaField::Fish { .. } => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct GameState {
    // todo handle class
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
                "ReceivedState should contain a 'start team' when converting to GameState"
                    .to_string(),
            );
        };

        let turn = if let Some(t) = recv_state.turn {
            t
        } else {
            return Err(
                "ReceivedState should contain 'turn' when converting to GameState".to_string(),
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
pub struct MoveChange {
    initial_square: (u8, u8),
    final_square: (u8, u8),
    // some if a fish was eaten
    fish_at_final: Option<PiranhaField>,
}

impl GameState {
    pub fn new_with_board(board: Board, start_team: Team) -> Self {
        GameState {
            class: None,
            start_team,
            turn: 0,
            board,
            last_move: None,
        }
    }
    pub fn current_team(&self) -> Team {
        if self.turn % 2 == 0 {
            match self.start_team {
                Team::One => Team::One,
                Team::Two => Team::Two,
            }
        } else {
            match self.start_team {
                Team::One => Team::Two,
                Team::Two => Team::One,
            }
        }
    }

    pub fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let team = self.current_team();

        for y in 0..10 {
            for x in 0..10 {
                let cell = self.board.get(x, y);

                if let PiranhaField::Fish { team: t, .. } = cell {
                    if *t != team {
                        continue;
                    }
                } else {
                    continue;
                }
                for dir in [
                    Direction::Left,
                    Direction::Right,
                    Direction::UP,
                    Direction::Down,
                    Direction::UpLeft,
                    Direction::UpRight,
                    Direction::DownLeft,
                    Direction::DownRight,
                ] {
                    let dis = self.board.count_fishes_on_axis(x, y, dir);
                    if Board::check_allowed(&self.board, x, y, dir, dis, team) {
                        moves.push(Move {
                            from: (x as u8, y as u8),
                            dir,
                        });
                    }
                }
            }
        }

        moves
    }

    /// assumes legal move
    pub fn make_move(&mut self, mv: Move) -> MoveChange {
        let dis = Board::count_fishes_on_axis(
            &self.board,
            mv.from.0 as usize,
            mv.from.1 as usize,
            mv.dir,
        );
        let goal_field = mv.to_goal_pos(dis);

        let (fx, fy) = (mv.from.0 as usize, mv.from.1 as usize);
        let (gx, gy) = (goal_field.0 as usize, goal_field.1 as usize);

        let field_at_goal = *self.board.get(gx, gy);
        let field_at_initial = *self.board.get(fx, fy);
        let fish_at_final = if !matches!(field_at_goal, PiranhaField::Empty) {
            Some(field_at_goal)
        } else {
            None
        };

        let removed = self.board.get_mut(fx, fy);
        *removed = PiranhaField::Empty;

        let goal = self.board.get_mut(gx, gy);
        *goal = field_at_initial;

        MoveChange {
            initial_square: mv.from,
            final_square: goal_field,
            fish_at_final,
        }
    }

    pub fn unmake_move(&mut self, change: MoveChange) {
        let (fx, fy) = (
            change.initial_square.0 as usize,
            change.initial_square.1 as usize,
        );
        let (gx, gy) = (
            change.final_square.0 as usize,
            change.final_square.1 as usize,
        );

        let moved_fish = *self.board.get(gx, gy);
        *self.board.get_mut(fx, fy) = moved_fish;

        match change.fish_at_final {
            Some(fish) => {
                *self.board.get_mut(gx, gy) = fish;
            }
            None => {
                *self.board.get_mut(gx, gy) = PiranhaField::Empty;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreTypes {
    Siegpunkte,
    Schwarmgröße,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationTypes {
    Sum,
    Average,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scores {
    pub score_type: ScoreTypes,
    pub value: u32,
    pub aggregation_type: AggregationTypes,
    pub relevant_for_ranking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Winner {
    pub team: Team,
    pub regular: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RoomMessage {
    Memento(Box<GameState>),
    Result(Box<GameResult>),
    WelcomeMessage,
    MoveRequest,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PreparedRoom {
    pub reservations: (String, String),
    pub room_id: String,
}
#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ComMessage {
    Joined(Joined),
    Left(Left),
    Room(Box<RoomMessage>),
    Admin(AdminMessage),
}
