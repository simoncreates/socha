use rand::{rngs::ThreadRng, Rng};
use socha::neutral::{PiranhaField, Size, Team};

fn get_random_field(rng: &mut ThreadRng) -> PiranhaField {
    let num = rng.random_range(0..100);
    if num < 30 {
        PiranhaField::Fish {
            team: rand_team(rng),
            size: rand_size(rng),
        }
    } else if num < 35 {
        PiranhaField::Squid
    } else {
        PiranhaField::Empty
    }
}

fn rand_team(rng: &mut ThreadRng) -> Team {
    if rng.random_bool(0.5) {
        Team::One
    } else {
        Team::Two
    }
}

fn rand_size(rng: &mut ThreadRng) -> Size {
    let num = rng.random_range(0..2);
    if num == 0 {
        Size::S
    } else if num == 1 {
        Size::M
    } else {
        Size::L
    }
}

#[cfg(test)]
pub mod tests {
    use rand::{rngs::ThreadRng, Rng};
    use socha::internal::{Board, GameState, MoveChange, Row};

    use crate::{get_random_field, rand_team};
    fn random_board(rng: &mut ThreadRng) -> Board {
        Board {
            rows: std::array::from_fn(|_| Row {
                fields: std::array::from_fn(|_| get_random_field(rng)),
            }),
        }
    }

    #[test]
    fn fuzz_make_unmake_games() {
        let mut rng = rand::rng();

        for game_idx in 0..100 {
            println!("Running fuzz game {}", game_idx);

            let board = random_board(&mut rng);
            let mut state = GameState::new_with_board(board, rand_team(&mut rng));

            let initial_board = state.board.clone();

            let mut move_stack: Vec<MoveChange> = Vec::new();
            for _ in 0..200 {
                let moves = state.possible_moves();

                if moves.is_empty() {
                    println!("stopping early");
                    break;
                }

                let mv = moves[rng.random_range(0..moves.len())];

                let change = state.make_move(mv);
                move_stack.push(change);
            }

            while let Some(change) = move_stack.pop() {
                state.unmake_move(change);
            }
            assert_eq!(state.board, initial_board, " board mismatch");
        }
    }
}
