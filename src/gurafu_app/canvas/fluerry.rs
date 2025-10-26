use petgraph::{Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};

use crate::gurafu_app::canvas::drawable::{arrow::Arrow, circle::Circle};

pub struct FluerryState {
    // Algorithm state
    algo_step: FluerryStep,
    stack: Vec<NodeIndex>,
    circuit: Vec<NodeIndex>,
    current_node: Option<NodeIndex>,
    graph_clone: StableGraph<Circle, Arrow>,

    // Step visualization state
    current_outgoing: Vec<(NodeIndex, petgraph::graph::EdgeIndex)>,
    current_idx: usize,
    next_candidate: Option<NodeIndex>,
    visited_edges: Vec<petgraph::graph::EdgeIndex>,
    step_explanation: String,
}

#[derive(Debug, Clone, Copy)]
enum FluerryStep {
    NotStarted,
    Initializing,
    CheckingOutgoing,
    ChoosingNext,
    Advancing,
    Backtracking,
    Completed,
    Failed,
}

impl FluerryState {
    pub fn new() -> Self {
        Self {
            algo_step: FluerryStep::NotStarted,
            stack: Vec::new(),
            circuit: Vec::new(),
            current_node: None,
            graph_clone: StableGraph::new(),
            current_outgoing: Vec::new(),
            current_idx: 0,
            next_candidate: None,
            visited_edges: Vec::new(),
            step_explanation: String::new(),
        }
    }

    pub fn restart_algoritm(&mut self) {
        self.algo_step = FluerryStep::NotStarted
    }

    pub fn step_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let cur_state = self.algo_step;

        match self.algo_step {
            FluerryStep::NotStarted => self.initialize_algorithm(graph),
            FluerryStep::Initializing => self.find_start_node(),
            FluerryStep::CheckingOutgoing => self.check_outgoing_edges(graph),
            FluerryStep::ChoosingNext => self.choose_next_edge(graph),
            FluerryStep::Advancing => self.advance_to_next(graph),
            FluerryStep::Backtracking => self.backtrack(),
            FluerryStep::Completed | FluerryStep::Failed => {
                // Algorithm finished, do nothing
                self.step_explanation = "Algorithm completed".to_string();
            }
        }

        //self.update_highlights();
        println!(
            "Step: {} - {}",
            format!("{:?}", cur_state),
            self.step_explanation
        );
    }

    fn initialize_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.reset_algorithm(graph);
        self.step_explanation =
            "Initializing algorithm - cloned graph and cleared state".to_string();
    }

    fn find_start_node(&mut self) {
        // Find a node with outgoing edges (for Eulerian path)
        // For Eulerian circuit, we can start anywhere with edges
        self.current_node = self.graph_clone.node_indices().find(|&node| {
            self.graph_clone
                .edges_directed(node, Direction::Outgoing)
                .count()
                > 0
        });

        if let Some(start_node) = self.current_node {
            self.circuit.push(start_node);
            self.algo_step = FluerryStep::CheckingOutgoing;
            self.step_explanation = format!("Starting from node {:?}", start_node);
        } else {
            self.algo_step = FluerryStep::Completed;
            self.step_explanation = "No suitable start node found - graph may be empty".to_string();
        }
    }

    fn check_outgoing_edges(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_step = FluerryStep::Failed;
                self.step_explanation = "No current node - algorithm failed".to_string();
                return;
            }
        };

        graph.node_weight_mut(current).unwrap().highlight_current();

        // Check if we're done
        if self.stack.is_empty()
            && self
                .graph_clone
                .edges_directed(current, Direction::Outgoing)
                .count()
                == 0
        {
            // Check if all edges are used
            if self.graph_clone.edge_count() == 0 {
                self.algo_step = FluerryStep::Completed;
                self.step_explanation = "Algorithm completed - Eulerian circuit found".to_string();
            } else {
                self.algo_step = FluerryStep::Failed;
                self.step_explanation =
                    "Algorithm failed - unused edges remain but no path forward".to_string();
            }
            return;
        }

        // Get current outgoing edges with their edge IDs
        self.current_outgoing = self
            .graph_clone
            .edges_directed(current, Direction::Outgoing)
            .map(|edge| (edge.target(), edge.id()))
            .collect();

        if self.current_outgoing.is_empty() {
            // No outgoing edges - need to backtrack
            self.algo_step = FluerryStep::Backtracking;
            self.step_explanation = "No outgoing edges available - backtracking".to_string();
        } else if self.current_outgoing.len() == 1 {
            // Only one choice - take it
            self.next_candidate = Some(self.current_outgoing[0].0);
            self.algo_step = FluerryStep::Advancing;
            self.step_explanation = "Only one outgoing edge - taking it".to_string();
        } else {
            // Multiple choices - need to choose non-bridge if possible
            self.current_idx = 0;
            self.algo_step = FluerryStep::ChoosingNext;
            self.step_explanation = format!(
                "Multiple outgoing edges ({}) - checking for non-bridge",
                self.current_outgoing.len()
            );
        }
    }

    fn choose_next_edge(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_step = FluerryStep::Failed;
                return;
            }
        };

        if self.current_idx >= self.current_outgoing.len() {
            // No non-bridge found, use first edge
            self.next_candidate = Some(self.current_outgoing[0].0);
            self.algo_step = FluerryStep::Advancing;
            self.step_explanation = "No non-bridge edge found - using first available".to_string();
            return;
        }

        let (candidate_node, edge_id) = self.current_outgoing[self.current_idx];

        // Highlight current candidate
        graph
            .node_weight_mut(candidate_node)
            .unwrap()
            .highlight_possibility();

        if self.is_bridge(current, candidate_node) {
            graph.edge_weight_mut(edge_id).unwrap().highlight_bridge();
            self.step_explanation = format!("Edge to {:?} is a bridge - skipping", candidate_node);
        } else {
            graph.edge_weight_mut(edge_id).unwrap().highlight_current();
            self.next_candidate = Some(candidate_node);
            self.algo_step = FluerryStep::Advancing;
            self.step_explanation = format!("Found non-bridge edge to {:?}", candidate_node);
            return;
        }

        self.current_idx += 1;

        if self.current_idx < self.current_outgoing.len() {
            self.step_explanation = format!(
                "Checking next candidate ({}/{})",
                self.current_idx + 1,
                self.current_outgoing.len()
            );
        }
    }

    fn advance_to_next(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_step = FluerryStep::Failed;
                return;
            }
        };

        let next = match self.next_candidate {
            Some(node) => node,
            None => {
                self.algo_step = FluerryStep::Failed;
                return;
            }
        };

        // Find and remove the edge
        if let Some(edge_id) = self.graph_clone.find_edge(current, next) {
            self.graph_clone.remove_edge(edge_id);
            self.visited_edges.push(edge_id);

            // Update original graph for visualization
            if let Some(edge_weight) = graph.edge_weight_mut(edge_id) {
                edge_weight.highlight_solution();
            }
        }

        // Move to next node
        self.stack.push(current);
        if let Some(cur_weight) = graph.node_weight_mut(self.current_node.unwrap()) {
            cur_weight.highlight_solution();
        }
        self.current_node = Some(next);
        self.circuit.push(next);

        self.current_outgoing.clear();
        self.next_candidate = None;
        self.algo_step = FluerryStep::CheckingOutgoing;

        self.step_explanation = format!("Moved from {:?} to {:?}", current, next);
    }

    fn backtrack(&mut self) {
        if let Some(prev_node) = self.stack.pop() {
            self.current_node = Some(prev_node);
            self.circuit.push(prev_node);
            self.algo_step = FluerryStep::CheckingOutgoing;
            self.step_explanation = format!("Backtracked to node {:?}", prev_node);
        } else {
            self.algo_step = FluerryStep::Failed;
            self.step_explanation = "Cannot backtrack - stack is empty".to_string();
        }
    }

    fn clear_highlights(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        for node in graph.node_weights_mut() {
            node.reset_highlight();
        }
        for edge in graph.edge_weights_mut() {
            edge.reset_highlight();
        }
    }

    fn reset_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.graph_clone = graph.clone();
        self.algo_step = FluerryStep::Initializing;
        self.stack.clear();
        self.circuit.clear();
        self.current_node = None;
        self.current_outgoing.clear();
        self.current_idx = 0;
        self.next_candidate = None;
        self.visited_edges.clear();
        self.step_explanation.clear();
        self.clear_highlights(graph);
    }

    // Improved bridge detection
    fn is_bridge(&self, u: NodeIndex, v: NodeIndex) -> bool {
        let mut temp_graph = self.graph_clone.clone();

        if let Some(edge_id) = temp_graph.find_edge(u, v) {
            temp_graph.remove_edge(edge_id);

            // Count reachable nodes from u in the original graph
            let original_reachable = petgraph::algo::dijkstra(&self.graph_clone, u, None, |_| 1);
            let after_remove_reachable = petgraph::algo::dijkstra(&temp_graph, u, None, |_| 1);

            // If the number of reachable nodes decreases, it's a bridge
            original_reachable.len() != after_remove_reachable.len()
        } else {
            false
        }
    }
}
