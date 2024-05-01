//! Graph functions.

use chess::{
    get_pawn_attacks, get_rank, BitBoard, Color, Piece, Square, ALL_SQUARES, EMPTY, NUM_SQUARES,
};
use petgraph::{
    algo::dijkstra,
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use super::moves_on_empty_board;
use crate::State;

pub struct MobilityGraph {
    graph: DiGraph<(), u32>,
    square_indices: [NodeIndex; NUM_SQUARES],
}

impl MobilityGraph {
    fn new() -> Self {
        let mut graph = DiGraph::<(), u32>::new();
        let square_indices = core::array::from_fn(|_| graph.add_node(()));
        Self {
            graph,
            square_indices,
        }
    }

    pub fn init(piece: Piece, color: Color) -> Self {
        let mut graph = Self::new();
        for source in ALL_SQUARES {
            if piece == Piece::Pawn {
                if BitBoard::from_square(source) & get_rank(color.to_my_backrank()) != EMPTY {
                    continue;
                }
                for target in get_pawn_attacks(source, color, !EMPTY) {
                    graph.add_edge(source, target, 1);
                }
            }
            for target in moves_on_empty_board(piece, color, source) {
                graph.add_edge(source, target, 0)
            }
        }
        graph
    }

    fn node(&self, square: Square) -> NodeIndex {
        self.square_indices[square.to_index()]
    }

    fn edge(&self, source: Square, target: Square) -> Option<EdgeIndex> {
        self.graph.find_edge(self.node(source), self.node(target))
    }

    fn add_edge(&mut self, source: Square, target: Square, weight: u32) {
        self.graph
            .add_edge(self.node(source), self.node(target), weight);
    }

    /// Makes sure the edge between the given squares disappears from the graph.
    /// Returns `true` iff this operation modifies the graph.
    fn remove_edge(&mut self, source: Square, target: Square) -> bool {
        match self.edge(source, target) {
            None => false,
            Some(edge) => {
                self.graph.remove_edge(edge);
                true
            }
        }
    }

    /// Removes all the given edges.
    fn remove_edges(&mut self, edges: &[EdgeIndex]) {
        for edge in edges.iter() {
            self.graph.remove_edge(*edge);
        }
    }

    /// Makes sure the graph does not have outgoing edges from the given node.
    /// Returns `true` iff this operation modifies the graph.
    fn remove_outgoing_edges(&mut self, source: Square) -> bool {
        let outgoing_edges: Vec<_> = self
            .graph
            .edges_directed(self.node(source), Outgoing)
            .map(|edge_ref| edge_ref.id())
            .collect();
        self.remove_edges(&outgoing_edges);
        !outgoing_edges.is_empty()
    }

    /// Makes sure the graph does not have incoming edges to the given node.
    /// Returns `true` iff this operation modifies the graph.
    fn remove_incoming_edges(&mut self, target: Square) -> bool {
        let incoming_edges: Vec<_> = self
            .graph
            .edges_directed(self.node(target), Incoming)
            .map(|edge_ref| edge_ref.id())
            .collect();
        self.remove_edges(&incoming_edges);
        !incoming_edges.is_empty()
    }

    pub fn distance(&self, source: Square, target: Square) -> Option<u32> {
        let node_map = dijkstra(&self.graph, self.node(source), None, |e| *e.weight());
        node_map.get(&self.node(target)).copied()
    }
}

/// The minimum number of captures necessary for the given piece of the
/// given color to go from `source` to `target`, according to the
/// current information about the position. If this function returns
/// `None`, the route from `source` to `target` is definitely
/// impossible. If this function returns `Some(n)`, at least `n`
/// captures are required (but this does not mean that it is possible
/// with exactly `n` captures).
///
/// Note that we also consider the possibility of the piece starting its
/// journey as a pawn, and having promoted.
pub fn distance_from_source(
    state: &State,
    source: Square,
    target: Square,
    piece: Piece,
    color: Color,
) -> Option<u32> {
    let piece_graph = &state.mobility.value[color.to_index()][piece.to_index()];
    // the distance without promotions
    let mut distance = piece_graph.distance(source, target);

    // consider the possibility of a promotion
    if piece != Piece::Pawn && piece != Piece::King {
        let pawn_graph = &state.mobility.value[color.to_index()][Piece::Pawn.to_index()];
        for promoting_square in get_rank(color.to_their_backrank()) {
            let d1 = pawn_graph.distance(source, promoting_square);
            let d2 = piece_graph.distance(promoting_square, target);
            if d1.is_some()
                && d2.is_some()
                && (distance.is_none() || d1.unwrap() + d2.unwrap() < distance.unwrap())
            {
                distance = Some(d1.unwrap() + d2.unwrap());
            }
        }
    }
    distance
}

/// The minimum number of captures necessary for the given piece of the
/// given color to go from `source` to `target`, according to the
/// current information about the position. If this function returns
/// `None`, the route from `source` to `target` is definitely
/// impossible. If this function returns `Some(n)`, at least `n`
/// captures are required (but this does not mean that it is possible
/// with exactly `n` captures).
///
/// If the piece is a pawn, it is allowed to promote in order to reach
/// the target.
pub fn distance_to_target(
    state: &State,
    source: Square,
    target: Square,
    piece: Piece,
    color: Color,
) -> Option<u32> {
    let piece_graph = &state.mobility.value[color.to_index()][piece.to_index()];
    // the distance without promotions
    let mut distance = piece_graph.distance(source, target);

    // if the piece is a pawn and can promote, we assume it can then reach the
    // target without further captures
    if piece == Piece::Pawn {
        for promoting_square in get_rank(color.to_their_backrank()) {
            let d = piece_graph.distance(source, promoting_square);
            if d.is_some() && (distance.is_none() || d.unwrap() < distance.unwrap()) {
                distance = d;
            }
        }
    }
    distance
}

#[cfg(test)]
mod tests {

    use chess::Board;
    use Color::*;
    use Piece::*;

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_init() {
        let king_mobility = MobilityGraph::init(King, White);
        assert_eq!(king_mobility.graph.edge_count(), 420);

        let queen_mobility = MobilityGraph::init(Queen, White);
        assert_eq!(queen_mobility.graph.edge_count(), 896 + 560);

        let rook_mobility = MobilityGraph::init(Rook, Black);
        assert_eq!(rook_mobility.graph.edge_count(), 896);

        let bishop_mobility = MobilityGraph::init(Bishop, Black);
        assert_eq!(bishop_mobility.graph.edge_count(), 560);

        let knight_mobility = MobilityGraph::init(Knight, White);
        assert_eq!(knight_mobility.graph.edge_count(), 336);

        let white_pawn_mobility = MobilityGraph::init(Pawn, White);
        assert_eq!(white_pawn_mobility.graph.edge_count(), 140);

        let black_pawn_mobility = MobilityGraph::init(Pawn, Black);
        assert_eq!(black_pawn_mobility.graph.edge_count(), 140);

        assert_eq!(white_pawn_mobility.distance(E2, C4), Some(2));
        assert_eq!(white_pawn_mobility.distance(E2, E4), Some(0));
        assert_eq!(white_pawn_mobility.distance(E2, F6), Some(1));
        assert_eq!(white_pawn_mobility.distance(E2, H4), None);
        assert_eq!(white_pawn_mobility.distance(E2, H5), Some(3));
    }

    #[test]
    fn test_distance_from_source() {
        let mut state = State::new(&Board::default());

        // a bishop on H5 cannot have come from C1, a dark square
        assert_eq!(distance_from_source(&state, C1, H5, Bishop, White), None);

        // but it may have come from B1, a light square, no captures needed
        assert_eq!(distance_from_source(&state, B1, H5, Bishop, White), Some(0));

        // it can also have come from B2, although it is a dark square, because
        // it could have been a promoted pawn, at least a capture is needed, though
        // to switch to a file with a light promoting square
        assert_eq!(distance_from_source(&state, B2, H5, Bishop, White), Some(1));

        // from E3, no captures are needed
        assert_eq!(distance_from_source(&state, E3, H5, Bishop, White), Some(0));

        // or from B2 if the bishop were Black (as B1 is light)
        assert_eq!(distance_from_source(&state, B2, H5, Bishop, Black), Some(0));

        // let us remove some graph connections
        state.mobility.value[White.to_index()][Bishop.to_index()].remove_outgoing_edges(E8);

        // now we cannot promote on E8 because we disconnected E8 from H5
        assert_eq!(distance_from_source(&state, E3, H5, Bishop, White), Some(2));

        // a black pawn on C3 can come from F7, but it takes 3 captures
        assert_eq!(distance_from_source(&state, F7, C3, Pawn, Black), Some(3));

        // of course, it cannot come from G8
        assert_eq!(distance_from_source(&state, G8, C3, Pawn, Black), None);

        // and it cannot come from H7, because it would not be a pawn after a promotion
        assert_eq!(distance_from_source(&state, H7, C3, Pawn, Black), None);

        // if we remove the connection E6 -> D5, it can still come from F7
        state.mobility.value[Black.to_index()][Pawn.to_index()].remove_edge(E6, D5);
        assert_eq!(distance_from_source(&state, F7, C3, Pawn, Black), Some(3));

        // but also removing E5 -> D4 will disconnect it from F7
        state.mobility.value[Black.to_index()][Pawn.to_index()].remove_edge(E5, D4);
        assert_eq!(distance_from_source(&state, F7, C3, Pawn, Black), None);
    }

    #[test]
    fn test_distance_to_target() {
        let mut state = State::new(&Board::default());

        // a queen should be able to go anywhere without captures
        assert_eq!(distance_to_target(&state, A1, H8, Queen, Black), Some(0));

        // a pawn too if it can promote on their original file
        assert_eq!(distance_to_target(&state, A2, C4, Pawn, White), Some(0));

        // even if we disallow A2 -> A3, it can still go A2 -> A4 in one go
        state.mobility.value[White.to_index()][Pawn.to_index()].remove_edge(A2, A3);
        assert_eq!(distance_to_target(&state, A2, C4, Pawn, White), Some(0));

        // but also removing A2 -> A4 will force the pawn to capture at least once
        state.mobility.value[White.to_index()][Pawn.to_index()].remove_edge(A2, A4);
        assert_eq!(distance_to_target(&state, A2, C4, Pawn, White), Some(1));

        // finally, if we disallow promotions on B8, it will take at least 2 captures
        state.mobility.value[White.to_index()][Pawn.to_index()].remove_incoming_edges(B8);
        assert_eq!(distance_to_target(&state, A2, C4, Pawn, White), Some(2));
    }
}
