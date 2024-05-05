//! Graph functions.

use std::cmp::min;

use chess::{
    get_pawn_attacks, get_rank, BitBoard, Color, Piece, Rank, Square, ALL_SQUARES, EMPTY,
    NUM_COLORS, NUM_PIECES, NUM_SQUARES,
};
use petgraph::{
    algo::{astar, dijkstra},
    graph::{DiGraph, EdgeIndex, EdgeReference, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use super::moves_on_empty_board;

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

    #[cfg(test)]
    /// Tells whether there exists an edge between the two given squares.
    pub fn exists_edge(&self, source: Square, target: Square) -> bool {
        self.edge(source, target).is_some()
    }

    /// Makes sure the edge between the given squares disappears from the graph.
    /// Returns `true` iff this operation modifies the graph.
    pub fn remove_edge(&mut self, source: Square, target: Square) -> bool {
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
    pub fn remove_outgoing_edges(&mut self, source: Square) -> bool {
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
    pub fn remove_incoming_edges(&mut self, target: Square) -> bool {
        let incoming_edges: Vec<_> = self
            .graph
            .edges_directed(self.node(target), Incoming)
            .map(|edge_ref| edge_ref.id())
            .collect();
        self.remove_edges(&incoming_edges);
        !incoming_edges.is_empty()
    }

    /// Makes sure the given node is disconnected from the rest of the graph.
    /// Returns `true` iff this operation modifies the graph.
    #[allow(dead_code)]
    pub fn remove_node_edges(&mut self, node: Square) -> bool {
        self.remove_outgoing_edges(node) || self.remove_incoming_edges(node)
    }

    pub fn distance(&self, source: Square, target: Square) -> Option<u32> {
        // switch to A*?
        let node_map = dijkstra(&self.graph, self.node(source), None, |e| *e.weight());
        node_map.get(&self.node(target)).copied()
    }

    /// Returns a pair including:
    /// - a `BitBoard` with all the squares where a capture must have taken
    ///   place for going from `source` to `target` in this mobility graph,
    /// - a lower bound in the number of captures to navigate from `source` to
    ///   `target`.
    ///
    /// Note that the lower bound may be greater than the BitBoard's popcnt,
    /// because, e.g. it may be possible that 2 captures are needed, while
    /// only 1 square can be proven to be certainly in the path.
    ///
    /// This function returns `None` if the route is impossible.
    pub fn forced_captures(&self, source: Square, target: Square) -> Option<(BitBoard, u32)> {
        let source = self.node(source);
        let finish = |n| n == self.node(target);
        match astar(&self.graph, source, finish, |e| *e.weight(), |_| 0) {
            None => None,
            Some((distance, path)) => {
                let mut forced = EMPTY;
                for node in path.iter().skip(1) {
                    // If after significantly increasing the weight of capturing edges that arrive
                    // to `node`, the distance from source to target increases by the same amount,
                    // it must be the case that `node` is an essential (capturing) square.

                    const DELTA: u32 = 1000;
                    let new_weights = |e: EdgeReference<u32, u32>| {
                        let mut weight = *e.weight();
                        if weight == 1 && e.target() == *node {
                            weight += DELTA;
                        }
                        weight
                    };
                    let node_map =
                        dijkstra(&self.graph, source, Some(self.node(target)), new_weights);
                    if let Some(new_distance) = node_map.get(&self.node(target)).copied() {
                        if new_distance == distance + DELTA {
                            forced |= BitBoard::from_square(ALL_SQUARES[node.index()])
                        }
                    }
                }
                Some((forced, distance))
            }
        }
    }
}

/// The minimum number of captures necessary for the given piece of the
/// given color to go from the starting square `origin` to `target`, according
/// to the current information about the position. If this function returns
/// `None`, the route from `origin` to `target` is definitely
/// impossible. If this function returns `Some(n)`, at least `n`
/// captures are required (but this does not mean that it is possible
/// with exactly `n` captures).
///
/// Note that if the origin square is in the (relative) 2nd rank, the pawn may
/// have to promote before becoming the desired piece.
pub fn distance_from_origin(
    mobility_graphs: &[[MobilityGraph; NUM_PIECES]; NUM_COLORS],
    origin: Square,
    target: Square,
    piece: Piece,
    color: Color,
) -> Option<u32> {
    let piece_graph = &mobility_graphs[color.to_index()][piece.to_index()];
    if (BitBoard::from_square(origin) & get_rank(color.to_second_rank())) == EMPTY
        || piece == Piece::Pawn
    {
        // the distance without promotions
        piece_graph.distance(origin, target)
    } else {
        // the distance after promoting
        let pawn_graph = &mobility_graphs[color.to_index()][Piece::Pawn.to_index()];
        let mut distance = None;
        for promoting_square in get_rank(color.to_their_backrank()) {
            let d1 = pawn_graph.distance(origin, promoting_square);
            let d2 = piece_graph.distance(promoting_square, target);
            if d1.is_some()
                && d2.is_some()
                && (distance.is_none() || d1.unwrap() + d2.unwrap() < distance.unwrap())
            {
                distance = Some(d1.unwrap() + d2.unwrap());
            }
        }
        distance
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
/// If the piece is a pawn, it is allowed to promote in order to reach
/// the target.
pub fn distance_to_target(
    mobility_graphs: &[[MobilityGraph; NUM_PIECES]; NUM_COLORS],
    source: Square,
    target: Square,
    piece: Piece,
    color: Color,
) -> Option<u32> {
    let piece_graph = &mobility_graphs[color.to_index()][piece.to_index()];
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

/// The squares where the piece that started the game on `origin` must have
/// captured enemy pieces in order to go from `origin` to `target`, with at most
/// `nb_allowed_captures` captures, according to the current information about
/// the position.
/// If `final_piece` is set, the piece that lands on `target` must
/// be of this type, and a promotion may need to take place.
/// If `final_piece = None`, a promotion may or may not have happened before
/// reaching `target`.
///
/// This function also returns the minimum number of captures necessary to
/// perform the journey as a second argument.
///
/// If the specified route is impossible, this function returns `EMPTY`.
pub fn tombs_to_target(
    mobility_graphs: &[[MobilityGraph; NUM_PIECES]; NUM_COLORS],
    origin: Square,
    target: Square,
    nb_allowed_captures: u32,
    final_piece: Option<Piece>,
) -> (BitBoard, u32) {
    let color = match origin.get_rank() {
        Rank::Second => Color::White,
        Rank::Seventh => Color::Black,
        // we only know how to derive non-trivial tombs information for pawns
        _ => return (EMPTY, 0),
    };
    let mut tombs = !EMPTY;
    let mut min_distance = 16;
    let pawn_graph = &mobility_graphs[color.to_index()][Piece::Pawn.to_index()];

    // the pawn goes directly to target
    if final_piece.is_none() || final_piece == Some(Piece::Pawn) {
        if let Some((path_tombs, distance)) = pawn_graph.forced_captures(origin, target) {
            if distance <= nb_allowed_captures {
                tombs &= path_tombs;
                min_distance = min(distance, min_distance);
            }
        }
    }

    // the pawn promotes before going to target
    if final_piece != Some(Piece::Pawn) {
        let candidate_promotion_pieces = match final_piece {
            // knights first, they are more likely to be able to reach any square after promotion
            None => vec![Piece::Knight, Piece::Queen, Piece::Rook, Piece::Bishop],
            Some(piece) => vec![piece],
        };
        for promoting_square in get_rank(color.to_their_backrank()) {
            if let Some((path_tombs, d1)) = pawn_graph.forced_captures(origin, promoting_square) {
                for piece in candidate_promotion_pieces.clone() {
                    let piece_graph = &mobility_graphs[color.to_index()][piece.to_index()];
                    let d2 = piece_graph.distance(promoting_square, target);
                    if d2.is_some() && d1 + d2.unwrap() <= nb_allowed_captures {
                        tombs &= path_tombs;
                        min_distance = min(d1 + d2.unwrap(), min_distance);
                        // the promotion piece is unimportant, we can stop now that a path was found
                        break;
                    }
                }
            }
        }
    }

    // if at this point tombs == !EMPTY, all routes were impossible, so return EMPTY
    if tombs == !EMPTY {
        return (EMPTY, min_distance);
    }

    (tombs, min_distance)
}

#[cfg(test)]
mod tests {

    use chess::ALL_PIECES;
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
    fn test_distance_from_origin() {
        let mut graphs = [
            core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::White)),
            core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::Black)),
        ];

        // a bishop on H5 cannot have come from C1, a dark square
        assert_eq!(distance_from_origin(&graphs, C1, H5, Bishop, White), None);

        // but it may have come from F1, a light square, no captures needed
        assert_eq!(
            distance_from_origin(&graphs, B1, H5, Bishop, White),
            Some(0)
        );

        // it can also have come from B2, although it is a dark square, because
        // it could have been a promoted pawn, at least a capture is needed though,
        // to switch to a file with a light promoting square
        assert_eq!(
            distance_from_origin(&graphs, B2, H5, Bishop, White),
            Some(1)
        );

        // or from B7 if the bishop were Black (as B1 is light)
        assert_eq!(
            distance_from_origin(&graphs, B7, H5, Bishop, Black),
            Some(0)
        );

        // let us remove some graph connections
        graphs[White.to_index()][Bishop.to_index()].remove_outgoing_edges(A8);
        graphs[White.to_index()][Bishop.to_index()].remove_outgoing_edges(C8);

        // now we cannot promote on A8 or C8, it has to be E8 which takes 3 captures
        assert_eq!(
            distance_from_origin(&graphs, B2, H5, Bishop, White),
            Some(3)
        );

        // a black pawn on C3 can come from F7, but it takes 3 captures
        assert_eq!(distance_from_origin(&graphs, F7, C3, Pawn, Black), Some(3));

        // of course, it cannot come from G8
        assert_eq!(distance_from_origin(&graphs, G8, C3, Pawn, Black), None);

        // and it cannot come from H7, because it would not be a pawn after a promotion
        assert_eq!(distance_from_origin(&graphs, H7, C3, Pawn, Black), None);

        // if we remove the connection E6 -> D5, it can still come from F7
        graphs[Black.to_index()][Pawn.to_index()].remove_edge(E6, D5);
        assert_eq!(distance_from_origin(&graphs, F7, C3, Pawn, Black), Some(3));

        // but also removing E5 -> D4 will disconnect it from F7
        graphs[Black.to_index()][Pawn.to_index()].remove_edge(E5, D4);
        assert_eq!(distance_from_origin(&graphs, F7, C3, Pawn, Black), None);
    }

    #[test]
    fn test_distance_to_target() {
        let mut graphs = [
            core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::White)),
            core::array::from_fn(|i| MobilityGraph::init(ALL_PIECES[i], Color::Black)),
        ];

        // a queen should be able to go anywhere without captures
        assert_eq!(distance_to_target(&graphs, A1, H8, Queen, Black), Some(0));

        // a pawn too if it can promote on their original file
        assert_eq!(distance_to_target(&graphs, A2, C4, Pawn, White), Some(0));

        // even if we disallow A2 -> A3, it can still go A2 -> A4 in one go
        graphs[White.to_index()][Pawn.to_index()].remove_edge(A2, A3);
        assert_eq!(distance_to_target(&graphs, A2, C4, Pawn, White), Some(0));

        // but also removing A2 -> A4 will force the pawn to capture at least once
        graphs[White.to_index()][Pawn.to_index()].remove_edge(A2, A4);
        assert_eq!(distance_to_target(&graphs, A2, C4, Pawn, White), Some(1));

        // finally, if we also disallow promotions on B8, it takes at least 2 captures
        graphs[White.to_index()][Pawn.to_index()].remove_incoming_edges(B8);
        assert_eq!(distance_to_target(&graphs, A2, C4, Pawn, White), Some(2));
    }
}
