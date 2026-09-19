//! Mobility graphs.
//!
//! A mobility graph records the moves that a piece may still have made: its
//! nodes are the squares of the board and its edges the moves that have not
//! been ruled out yet, each of them either quiet or capturing. Rules remove
//! edges as they learn, and the analysis queries the graph about the routes
//! that remain.

use chess::{
    get_bishop_moves, get_pawn_attacks, get_pawn_quiets, get_rank, get_rook_moves, BitBoard, Color,
    Piece, Square, ALL_SQUARES, EMPTY, NUM_SQUARES,
};

use super::moves_on_empty_board;

/// The moves that a piece of a given type and color may still have made,
/// as the squares reachable from each square, by a quiet move or by a
/// capture.
pub struct MobilityGraph {
    piece: Piece,
    color: Color,
    quiet: [BitBoard; NUM_SQUARES],
    captures: [BitBoard; NUM_SQUARES],
}

impl MobilityGraph {
    /// The graph of all the moves of a piece of the given type and color on
    /// an empty board (pawns also capture, which no other piece does in a
    /// way that changes its route).
    pub fn init(piece: Piece, color: Color) -> Self {
        let mut graph = MobilityGraph {
            piece,
            color,
            quiet: [EMPTY; NUM_SQUARES],
            captures: [EMPTY; NUM_SQUARES],
        };
        for source in ALL_SQUARES {
            if piece == Piece::Pawn {
                if BitBoard::from_square(source) & get_rank(color.to_my_backrank()) != EMPTY {
                    continue;
                }
                graph.captures[source.to_index()] = get_pawn_attacks(source, color, !EMPTY);
            }
            graph.quiet[source.to_index()] = moves_on_empty_board(piece, color, source);
        }
        graph
    }

    /// The moves from the given square, capturing or not.
    fn moves_from(&self, source: Square) -> BitBoard {
        self.quiet[source.to_index()] | self.captures[source.to_index()]
    }

    /// Whether there exists a move between the two given squares.
    pub fn exists_edge(&self, source: Square, target: Square) -> bool {
        self.moves_from(source) & BitBoard::from_square(target) != EMPTY
    }

    /// Makes sure the move between the given squares disappears from the
    /// graph. Returns `true` iff this operation modifies the graph.
    pub fn remove_edge(&mut self, source: Square, target: Square) -> bool {
        let existed = self.exists_edge(source, target);
        let target = BitBoard::from_square(target);
        self.quiet[source.to_index()] &= !target;
        self.captures[source.to_index()] &= !target;
        existed
    }

    /// Makes sure the graph does not have moves from the given square.
    /// Returns `true` iff this operation modifies the graph.
    pub fn remove_outgoing_edges(&mut self, source: Square) -> bool {
        let existed = self.moves_from(source) != EMPTY;
        self.quiet[source.to_index()] = EMPTY;
        self.captures[source.to_index()] = EMPTY;
        existed
    }

    /// Makes sure the graph does not have moves to the given square.
    /// Returns `true` iff this operation modifies the graph.
    pub fn remove_incoming_edges(&mut self, target: Square) -> bool {
        let mut existed = false;
        for source in ALL_SQUARES {
            existed |= self.remove_edge(source, target);
        }
        existed
    }

    /// Makes sure no move goes into, out of, or through any of the given
    /// squares. Returns `true` iff this operation modifies the graph.
    pub fn remove_edges_touching(&mut self, squares: BitBoard) -> bool {
        let mut modified = false;
        for source in ALL_SQUARES {
            let i = source.to_index();
            let (quiet, captures) = if BitBoard::from_square(source) & squares != EMPTY {
                (EMPTY, EMPTY)
            } else {
                let allowed = match self.piece {
                    Piece::Rook => get_rook_moves(source, squares),
                    Piece::Bishop => get_bishop_moves(source, squares),
                    Piece::Queen => {
                        get_rook_moves(source, squares) | get_bishop_moves(source, squares)
                    }
                    Piece::Pawn => get_pawn_quiets(source, self.color, squares),
                    Piece::King | Piece::Knight => !EMPTY,
                } & !squares;
                (self.quiet[i] & allowed, self.captures[i] & !squares)
            };
            modified |= quiet != self.quiet[i] || captures != self.captures[i];
            self.quiet[i] = quiet;
            self.captures[i] = captures;
        }
        modified
    }

    /// The squares from which there exists a move to the given `target`.
    pub fn predecessors(&self, target: Square) -> BitBoard {
        let target = BitBoard::from_square(target);
        ALL_SQUARES
            .into_iter()
            .filter(|source| self.moves_from(*source) & target != EMPTY)
            .fold(EMPTY, |acc, source| acc | BitBoard::from_square(source))
    }

    /// Makes sure the given square is disconnected from the rest of the
    /// graph. Returns `true` iff this operation modifies the graph.
    #[allow(dead_code)]
    pub fn remove_node_edges(&mut self, node: Square) -> bool {
        self.remove_outgoing_edges(node) | self.remove_incoming_edges(node)
    }

    /// The least number of captures on a route from `source` to `target`,
    /// if there exists one.
    #[cfg(test)]
    pub fn distance(&self, source: Square, target: Square) -> Option<u32> {
        let distance = self.distances_from_source(source)[target.to_index()];
        (distance < UNREACHABLE).then_some(distance as u32)
    }

    /// The squares that can be reached from the given `source`, this one
    /// included.
    pub fn reachable_from_source(&self, source: Square) -> BitBoard {
        let mut reachable = BitBoard::from_square(source);
        let mut frontier = reachable;
        while frontier != EMPTY {
            let discovered = frontier.fold(EMPTY, |acc, square| acc | self.moves_from(square));
            frontier = discovered & !reachable;
            reachable |= frontier;
        }
        reachable
    }

    /// The least number of captures on a route from `source` to each square
    /// ([`UNREACHABLE`] for the squares that cannot be reached).
    pub fn distances_from_source(&self, source: Square) -> [u8; NUM_SQUARES] {
        let mut distances = [UNREACHABLE; NUM_SQUARES];
        let mut visited = EMPTY;
        let mut level = BitBoard::from_square(source);
        for captures in 0..UNREACHABLE {
            level &= !visited;
            if level == EMPTY {
                break;
            }
            // The squares of a level are those reached with the same number
            // of captures, which the quiet moves preserve.
            let mut frontier = level;
            while frontier != EMPTY {
                let discovered =
                    frontier.fold(EMPTY, |acc, square| acc | self.quiet[square.to_index()]);
                frontier = discovered & !level & !visited;
                level |= frontier;
            }
            for square in level {
                distances[square.to_index()] = captures;
            }
            visited |= level;
            level = level.fold(EMPTY, |acc, square| acc | self.captures[square.to_index()]);
        }
        distances
    }

    /// The squares where a capture must have taken place on the way from
    /// `source` to each square, with at most `allowed_nb_captures` of them
    /// (`EMPTY` for the squares that cannot be reached within that many).
    ///
    /// A square is *forced* for a target when every route to that target
    /// within the budget arrives at the square by a capture.
    pub fn forced_captures_from(
        &self,
        source: Square,
        allowed_nb_captures: u8,
    ) -> [BitBoard; NUM_SQUARES] {
        // The routes of interest are those of at most `allowed_nb_captures`
        // captures, so it is enough to know, for every square and every
        // number of captures spent to get there, the squares that all such
        // routes capture on: `current[square]` for the level being explored,
        // `None` for the squares that no such route reaches. A square is
        // forced for a target when the routes force it whatever the number
        // of captures they spend, so the levels are combined by intersection
        // into `forced` as they complete.
        let budget = (allowed_nb_captures as usize).min(UNREACHABLE as usize - 1);
        let mut forced = [None; NUM_SQUARES];
        let mut current = [None; NUM_SQUARES];
        current[source.to_index()] = Some(EMPTY);
        // the squares reached at the current level
        let mut reached = BitBoard::from_square(source);
        for spent in 0..=budget {
            // Quiet moves keep the number of captures, so the routes that
            // spend `spent` of them are complete once these are exhausted.
            let mut pending = reached;
            while pending != EMPTY {
                let square = pending.to_square();
                pending &= !BitBoard::from_square(square);
                let routes = current[square.to_index()].unwrap();
                for target in self.quiet[square.to_index()] {
                    if update(&mut current[target.to_index()], routes) {
                        pending |= BitBoard::from_square(target);
                        reached |= BitBoard::from_square(target);
                    }
                }
            }
            for square in reached {
                update(
                    &mut forced[square.to_index()],
                    current[square.to_index()].unwrap(),
                );
            }
            if spent == budget {
                break;
            }
            let mut next = [None; NUM_SQUARES];
            let mut next_reached = EMPTY;
            for square in reached {
                let routes = current[square.to_index()].unwrap();
                for target in self.captures[square.to_index()] {
                    update(
                        &mut next[target.to_index()],
                        routes | BitBoard::from_square(target),
                    );
                    next_reached |= BitBoard::from_square(target);
                }
            }
            if next_reached == EMPTY {
                break;
            }
            current = next;
            reached = next_reached;
        }
        core::array::from_fn(|square| forced[square].unwrap_or(EMPTY))
    }
}

/// The distance recorded for the squares that cannot be reached.
const UNREACHABLE: u8 = 16;

/// Adds the routes to those known to arrive at a square, which force what
/// all of them force. Returns `true` iff this operation modifies them.
fn update(known: &mut Option<BitBoard>, routes: BitBoard) -> bool {
    let updated = known.map_or(routes, |known| known & routes);
    let modified = *known != Some(updated);
    *known = Some(updated);
    modified
}

#[cfg(test)]
mod tests {

    use Color::*;
    use Piece::*;

    use super::*;
    use crate::utils::*;

    /// The number of moves in the graph.
    fn edge_count(graph: &MobilityGraph) -> usize {
        ALL_SQUARES
            .into_iter()
            .map(|square| graph.moves_from(square).popcnt() as usize)
            .sum()
    }

    #[test]
    fn test_init() {
        let king_mobility = MobilityGraph::init(King, White);
        assert_eq!(edge_count(&king_mobility), 420);

        let queen_mobility = MobilityGraph::init(Queen, White);
        assert_eq!(edge_count(&queen_mobility), 896 + 560);

        let rook_mobility = MobilityGraph::init(Rook, Black);
        assert_eq!(edge_count(&rook_mobility), 896);

        let bishop_mobility = MobilityGraph::init(Bishop, Black);
        assert_eq!(edge_count(&bishop_mobility), 560);

        let knight_mobility = MobilityGraph::init(Knight, White);
        assert_eq!(edge_count(&knight_mobility), 336);

        let white_pawn_mobility = MobilityGraph::init(Pawn, White);
        assert_eq!(edge_count(&white_pawn_mobility), 140);

        let black_pawn_mobility = MobilityGraph::init(Pawn, Black);
        assert_eq!(edge_count(&black_pawn_mobility), 140);

        assert_eq!(white_pawn_mobility.distance(E2, C4), Some(2));
        assert_eq!(white_pawn_mobility.distance(E2, E4), Some(0));
        assert_eq!(white_pawn_mobility.distance(E2, F6), Some(1));
        assert_eq!(white_pawn_mobility.distance(E2, H4), None);
        assert_eq!(white_pawn_mobility.distance(E2, H5), Some(3));
    }

    #[test]
    fn test_remove_edges() {
        let mut graph = MobilityGraph::init(Pawn, White);
        assert!(graph.remove_edge(A2, A3));
        assert!(!graph.remove_edge(A2, A3));
        assert!(graph.remove_outgoing_edges(H7));
        assert!(!graph.exists_edge(H7, G8));
        assert!(!graph.exists_edge(H7, H8));
        assert!(!graph.remove_outgoing_edges(H7));
        assert!(graph.remove_incoming_edges(D4));
        assert_eq!(graph.predecessors(D4), EMPTY);
        assert!(!graph.remove_incoming_edges(D4));
        assert!(graph.remove_node_edges(E4));
        assert_eq!(graph.moves_from(E4), EMPTY);
        assert_eq!(graph.predecessors(E4), EMPTY);
        assert_eq!(edge_count(&graph), 140 - 1 - 2 - 4 - 3 - 4);
    }

    #[test]
    fn test_forced_captures() {
        let mut graph = MobilityGraph::init(Pawn, White);
        let forced = |graph: &MobilityGraph, target: Square, budget: u8| {
            graph.forced_captures_from(E2, budget)[target.to_index()]
        };
        // both e3xd4 and exd3-d4 are possible, nothing is forced
        assert_eq!(forced(&graph, D4, 1), EMPTY);
        // the only route to d3 captures on d3
        assert_eq!(forced(&graph, D3, 1), BitBoard::from_square(D3));
        // squares that cannot be reached within the budget
        assert_eq!(forced(&graph, D3, 0), EMPTY);
        assert_eq!(forced(&graph, H4, 3), EMPTY);

        graph.remove_edge(E2, E3);
        // the only route to d4 with at most 1 capture is exd3-d4
        assert_eq!(forced(&graph, D4, 1), BitBoard::from_square(D3));
        assert_eq!(forced(&graph, D4, 2), BitBoard::from_square(D3));
        assert_eq!(forced(&graph, F4, 1), BitBoard::from_square(F3));
        // e2-e4 is still available, so nothing is forced to reach e4
        assert_eq!(forced(&graph, E4, 2), EMPTY);
        // exd3xe4 and exf3xe4 only agree on the capture on e4
        graph.remove_edge(E2, E4);
        assert_eq!(forced(&graph, E4, 2), BitBoard::from_square(E4));
        graph.remove_edge(E2, F3);
        assert_eq!(
            forced(&graph, E4, 2),
            BitBoard::from_square(D3) | BitBoard::from_square(E4)
        );
        assert_eq!(forced(&graph, E4, 1), EMPTY);
    }
}
