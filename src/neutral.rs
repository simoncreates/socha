use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::incoming::ReceivedLastMove;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    UP,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}
impl TryFrom<&str> for Direction {
    type Error = String;
    fn try_from(dir_str: &str) -> Result<Self, Self::Error> {
        let dir = match dir_str {
            "UP" => Direction::UP,
            "UP_RIGHT" => Direction::UpRight,
            "RIGHT" => Direction::Right,
            "DOWN_RIGHT" => Direction::DownRight,
            "DOWN" => Direction::Down,
            "DOWN_LEFT" => Direction::DownLeft,
            "LEFT" => Direction::Left,
            "UP_LEFT" => Direction::UpLeft,
            _ => {
                return Err(format!("unknown direction token '{}'", dir_str));
            }
        };
        Ok(dir)
    }
}

impl Direction {
    pub fn to_delta(&self) -> (i32, i32) {
        match self {
            Direction::UP => (0, 1),
            Direction::UpRight => (1, 1),
            Direction::Right => (1, 0),
            Direction::DownRight => (1, -1),
            Direction::Down => (0, -1),
            Direction::DownLeft => (-1, -1),
            Direction::Left => (-1, 0),
            Direction::UpLeft => (-1, 1),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::UP => write!(f, "UP"),
            Direction::UpRight => write!(f, "UP_RIGHT"),
            Direction::Right => write!(f, "RIGHT"),
            Direction::DownRight => write!(f, "DOWN_RIGHT"),
            Direction::Down => write!(f, "DOWN"),
            Direction::DownLeft => write!(f, "DOWN_LEFT"),
            Direction::Left => write!(f, "LEFT"),
            Direction::UpLeft => write!(f, "UP_LEFT"),
        }
    }
}

impl TryFrom<(u8, u8, u8, u8)> for Direction {
    type Error = String;

    fn try_from(t: (u8, u8, u8, u8)) -> Result<Self, Self::Error> {
        let (x1, y1, x2, y2) = t;
        let dx = x2 as i32 - x1 as i32;
        let dy = y2 as i32 - y1 as i32;

        if dx == 0 && dy == 0 {
            return Err("no movement provided".to_string());
        }
        let step_x = dx.signum();
        let step_y = dy.signum();

        let dir = match (step_x, step_y) {
            (0, -1) => Direction::UP,
            (1, -1) => Direction::UpRight,
            (1, 0) => Direction::Right,
            (1, 1) => Direction::DownRight,
            (0, 1) => Direction::Down,
            (-1, 1) => Direction::DownLeft,
            (-1, 0) => Direction::Left,
            (-1, -1) => Direction::UpLeft,
            _ => return Err(format!("unsupported delta dx={}, dy={}", dx, dy)),
        };

        Ok(dir)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: (u8, u8),
    pub dir: Direction,
}

impl TryFrom<&ReceivedLastMove> for Move {
    type Error = String;
    fn try_from(recv_last_move: &ReceivedLastMove) -> Result<Self, Self::Error> {
        let from = if let Some(m) = &recv_last_move.from {
            (m.x, m.y)
        } else {
            return Err(
                "ReceivedLastMove should contain 'from' when converting to internal::Move"
                    .to_string(),
            );
        };

        let dir = if let Some(d) = &recv_last_move.direction {
            Direction::try_from(d.value.as_ref())?
        } else {
            return Err(
                "ReceivedLastMove should contain 'direction' when converting to internal::Move"
                    .to_string(),
            );
        };
        Ok(Self { from, dir })
    }
}

impl Move {
    pub fn to_goal_pos(&self, dis: u8) -> (u8, u8) {
        let (dx, dy) = self.dir.to_delta();

        let mut gx = self.from.0 as i32 + dx * dis as i32;
        let mut gy = self.from.1 as i32 + dy * dis as i32;

        gx = gx.clamp(0, 9);
        gy = gy.clamp(0, 9);

        (gx as u8, gy as u8)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum Team {
    #[default]
    One,
    Two,
}

impl Team {
    pub fn opponent(&self) -> Team {
        match *self {
            Team::One => Team::Two,
            Team::Two => Team::One,
        }
    }
}

impl TryFrom<&str> for Team {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ONE" => Ok(Team::One),
            "TWO" => Ok(Team::Two),
            other => Err(format!("unknown team token '{}'", other)),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Team::One => write!(f, "ONE"),
            Team::Two => write!(f, "TWO"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    S,
    M,
    L,
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Size::S => write!(f, "S"),
            Size::M => write!(f, "M"),
            Size::L => write!(f, "L"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
pub enum PiranhaField {
    #[default]
    Empty,
    Squid,
    Fish {
        team: Team,
        size: Size,
    },
}
impl fmt::Display for PiranhaField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PiranhaField::Empty => write!(f, "EMPTY"),
            PiranhaField::Squid => write!(f, "SQUID"),
            PiranhaField::Fish { team, size } => write!(f, "{}_{}", team, size),
        }
    }
}

impl FromStr for PiranhaField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s == "EMPTY" {
            return Ok(PiranhaField::Empty);
        }
        if s == "SQUID" {
            return Ok(PiranhaField::Squid);
        }
        let mut parts = s.split('_');
        let team_part = parts
            .next()
            .ok_or_else(|| format!("missing team part in '{}'", s))?;
        let size_part = parts
            .next()
            .ok_or_else(|| format!("missing size part in '{}'", s))?;

        if parts.next().is_some() {
            return Err(format!("too many parts in '{}'", s));
        }

        let team = match team_part {
            "ONE" => Team::One,
            "TWO" => Team::Two,
            other => {
                return Err(format!("unknown team token '{}'", other));
            }
        };

        let size = match size_part {
            "S" => Size::S,
            "M" => Size::M,
            "L" => Size::L,
            other => {
                return Err(format!("unknown size token '{}'", other));
            }
        };

        Ok(PiranhaField::Fish { team, size })
    }
}
