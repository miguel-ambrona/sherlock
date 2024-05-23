//! Parity rule.
//!
//! If the parity of the number of moves by every piece can be determined,
//! then the turn can also be determined. If the turn is not the expected one,
//! the position must be illegal.

use std::collections::HashMap;

use chess::{get_rank, BitBoard, Board, Color, Square, EMPTY};

use super::{Analysis, Rule};
use crate::{
    rules::ALL_ORIGINS,
    utils::{origin_color, LIGHT_SQUARES},
    Legality,
};

#[derive(Debug)]
pub struct ParityRule {
    mobility_counter: usize,
    destinies_counter: usize,
}

impl Rule for ParityRule {
    fn new() -> Self {
        ParityRule {
            mobility_counter: 0,
            destinies_counter: 0,
        }
    }

    fn update(&mut self, analysis: &Analysis) {
        self.mobility_counter = analysis.mobility.counter();
        self.destinies_counter = analysis.destinies.counter();
    }

    fn is_applicable(&self, analysis: &Analysis) -> bool {
        self.mobility_counter != analysis.mobility.counter()
            || self.destinies_counter != analysis.destinies.counter()
    }

    fn apply(&self, analysis: &mut Analysis) -> bool {
        let mut parity_nb_moves = 0;
        let mut origins = ALL_ORIGINS;

        // consider the parity of knight moves if totally determined
        for (bi, gi) in [(Square::B1, Square::G1), (Square::B8, Square::G8)] {
            if analysis.destinies(bi).popcnt() == 2 {
                if analysis.destinies(bi) != analysis.destinies(gi) {
                    return false;
                }
                origins &= !BitBoard::from_square(bi);
                origins &= !BitBoard::from_square(gi);
                parity_nb_moves += (analysis.destinies(bi) & LIGHT_SQUARES).popcnt();
            }
        }

        // perform a first pass to verify if it is worth applying the parity check
        for origin in origins {
            if analysis.is_steady(origin) {
                origins &= !BitBoard::from_square(origin);
                continue;
            }

            if analysis.destinies(origin).popcnt() != 1 {
                return false;
            }

            // missing pawns that may have promoted spoil the parity argument
            let color = origin_color(origin);
            if origin.get_rank() == color.to_second_rank()
                && !analysis.is_definitely_on_the_board(origin)
                && analysis.reachable(origin) & get_rank(color.to_their_backrank()) != EMPTY
            {
                return false;
            }
        }

        // check if the parity of the number of moves by every piece can be determined
        for origin in origins {
            match path_parity(analysis, origin, analysis.destinies(origin).to_square()) {
                None => return false,
                Some(n) => parity_nb_moves += n,
            }
        }

        if analysis.board.side_to_move() == Color::Black {
            parity_nb_moves += 1;
        }

        if parity_nb_moves % 2 == 1 {
            analysis.result = Some(Legality::Illegal);
        }

        false
    }
}

// Returns `Some n` if all paths to `target` by the piece which started the game
// in `origin`, from its starting square, require a number of moves whose parity
// is unique (in which case it coincides with the parity of `n`). Returns `None`
// if there exist paths of both parities or no paths at all.
fn path_parity(analysis: &Analysis, origin: Square, target: Square) -> Option<u32> {
    // we try to find a 2-coloring of the connected component of `target` (with
    // reversed arrows) that is reachable from `origin`; this function returns
    // `Some n` if such 2-coloring exists, in that case `n = 0` if the colors of
    // `source` and `target` are the same and `n = 1` otherwise
    debug_assert!(BitBoard::from_square(origin) & ALL_ORIGINS != EMPTY);
    let piece = Board::default().piece_on(origin).unwrap();
    let color = origin_color(origin);
    let mobility = &analysis.mobility.value[color.to_index()][piece.to_index()];
    let reachable_from_origin = analysis.reachable(origin);
    if BitBoard::from_square(target) & reachable_from_origin == EMPTY {
        return None;
    }
    let mut coloring = HashMap::new();
    let mut current_color = true;
    let mut current_nodes = BitBoard::from_square(target);
    loop {
        let mut new_nodes = EMPTY;
        for node in current_nodes.into_iter() {
            match coloring.get(&node) {
                None => {
                    coloring.insert(node, current_color);
                    new_nodes |= mobility.predecessors(node) & reachable_from_origin;
                }
                Some(color) => {
                    if *color != current_color {
                        return None;
                    }
                }
            }
        }

        if new_nodes == EMPTY {
            return coloring.get(&origin).map(|b| if *b { 0 } else { 1 });
        }

        current_nodes = new_nodes;
        current_color = !current_color;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_path_parity() {
        let mut analysis = Analysis::new(&Board::default());

        // white pawns
        assert_eq!(path_parity(&analysis, C2, C3), Some(1));
        assert_eq!(path_parity(&analysis, C2, D3), Some(1));
        assert_eq!(path_parity(&analysis, C2, C4), None);
        // black pawns
        assert_eq!(path_parity(&analysis, C7, C2), None);
        // remove C7 manually, as we did not perform the actual analysis on reachable
        // squares
        analysis.reachable.value[Square::A7.to_index()] &= !BitBoard::from_square(C7);
        assert_eq!(path_parity(&analysis, A7, C5), Some(0));
        // knights
        assert_eq!(path_parity(&analysis, B1, A1), Some(1));
        assert_eq!(path_parity(&analysis, G8, E4), Some(0));
        // bishops
        assert_eq!(path_parity(&analysis, C8, D7), None);
    }
}
