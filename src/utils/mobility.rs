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

    /// The squares that can be reached from the given `source`, this one
    /// included.
    pub fn reachable_from_source(&self, source: Square) -> BitBoard {
        // (a traversal, since the weights are irrelevant here)
        let mut reachable = BitBoard::from_square(source);
        let mut pending = vec![self.node(source)];
        while let Some(node) = pending.pop() {
            for successor in self.graph.neighbors_directed(node, Outgoing) {
                let square = BitBoard::from_square(ALL_SQUARES[successor.index()]);
                if reachable & square == EMPTY {
                    reachable |= square;
                    pending.push(successor);
                }
            }
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

    /// The squares where a capture must have taken place for going from
    /// `source` to each target in this mobility graph, with at most
    /// `allowed_nb_captures` (`EMPTY` for the targets that cannot be reached
    /// within that many).
    ///
    /// A square is *forced* for a target when every route to it within the
    /// budget arrives at that square by a capturing edge.
    pub fn forced_captures_from(
        &self,
        source: Square,
        allowed_nb_captures: u8,
    ) -> [BitBoard; NUM_SQUARES] {
        // The routes of interest are those of at most `allowed_nb_captures`
        // capturing edges, so it is enough to know, for every square and
        // every number of captures spent to get there, which squares all
        // such routes capture on: `forced[n][square]`. A square that no such
        // route reaches is marked as unvisited (`None`).
        let budget = allowed_nb_captures as usize;
        let mut forced: Vec<[Option<BitBoard>; NUM_SQUARES]> =
            vec![[None; NUM_SQUARES]; budget + 1];
        forced[0][source.to_index()] = Some(EMPTY);
        for spent in 0..=budget {
            // (a capturing edge moves a square to the next number of
            // captures, so the routes of `spent` captures are complete once
            // the quiet edges have been followed to exhaustion)
            let mut pending: Vec<Square> = ALL_SQUARES
                .into_iter()
                .filter(|s| forced[spent][s.to_index()].is_some())
                .collect();
            while let Some(square) = pending.pop() {
                let routes = forced[spent][square.to_index()].unwrap();
                for edge in self.graph.edges_directed(self.node(square), Outgoing) {
                    let target = ALL_SQUARES[edge.target().index()];
                    let capture = *edge.weight() == 1;
                    if capture && spent == budget {
                        continue;
                    }
                    let (level, routes) = match capture {
                        true => (spent + 1, routes | BitBoard::from_square(target)),
                        false => (spent, routes),
                    };
                    let known = &mut forced[level][target.to_index()];
                    let updated = match known {
                        // The routes that arrive with the same number of
                        // captures force what all of them force.
                        Some(previous) => *previous & routes,
                        None => routes,
                    };
                    if *known != Some(updated) {
                        *known = Some(updated);
                        if !capture {
                            pending.push(target);
                        }
                    }
                }
            }
        }
        // A square is forced for a target when all the routes force it,
        // whatever the number of captures they spend.
        core::array::from_fn(|i| {
            forced
                .iter()
                .filter_map(|level| level[i])
                .fold(None, |all: Option<BitBoard>, routes| {
                    Some(all.map_or(routes, |all| all & routes))
                })
                .unwrap_or(EMPTY)
        })
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
