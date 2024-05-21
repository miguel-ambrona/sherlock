//! Graph functions.

use chess::{
    get_pawn_attacks, get_rank, BitBoard, Color, Piece, Square, ALL_SQUARES, EMPTY, NUM_SQUARES,
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

    /// The squares for which there exists an edge to the given `target`.
    pub fn predecessors(&self, target: Square) -> BitBoard {
        let mut neighbors = EMPTY;
        for node in self.graph.neighbors_directed(self.node(target), Incoming) {
            neighbors |= BitBoard::from_square(ALL_SQUARES[node.index()]);
        }
        neighbors
    }

    /// Makes sure the given node is disconnected from the rest of the graph.
    /// Returns `true` iff this operation modifies the graph.
    #[allow(dead_code)]
    pub fn remove_node_edges(&mut self, node: Square) -> bool {
        self.remove_outgoing_edges(node) || self.remove_incoming_edges(node)
    }

    #[cfg(test)]
    pub fn distance(&self, source: Square, target: Square) -> Option<u32> {
        let node_map = dijkstra(&self.graph, self.node(source), None, |e| *e.weight());
        node_map.get(&self.node(target)).copied()
    }

    pub fn reachable_from_source(&self, source: Square) -> BitBoard {
        let node_map = dijkstra(&self.graph, self.node(source), None, |e| *e.weight());
        let mut reachable = EMPTY;
        for key in node_map.keys() {
            reachable |= BitBoard::from_square(ALL_SQUARES[key.index()]);
        }
        reachable
    }

    pub fn distances_from_source(&self, source: Square) -> [u8; NUM_SQUARES] {
        let node_map = dijkstra(&self.graph, self.node(source), None, |e| *e.weight());
        let mut distances = [16; NUM_SQUARES];
        for (key, bound) in node_map.iter() {
            distances[ALL_SQUARES[key.index()].to_index()] = *bound as u8;
        }
        distances
    }

    /// Returns a `BitBoard` with all the squares where a capture must have
    /// taken place for going from `source` to `target` in this mobility
    /// graph.
    ///
    /// This function returns `EMPTY` if the route is impossible.
    pub fn forced_captures(&self, source: Square, target: Square) -> BitBoard {
        let source = self.node(source);
        let finish = |n| n == self.node(target);
        match astar(&self.graph, source, finish, |e| *e.weight(), |_| 0) {
            None => EMPTY,
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
                forced
            }
        }
    }
}

#[cfg(test)]
mod tests {

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
}
