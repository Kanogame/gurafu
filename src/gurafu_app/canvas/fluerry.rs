use petgraph::{Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};
use std::collections::{HashSet, VecDeque};

use crate::gurafu_app::canvas::{
    CanvasMessage,
    drawable::{arrow::Arrow, circle::Circle},
};

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

    // Tracking for better highlight cleanup
    highlighted_candidates: Vec<NodeIndex>,
    highlighted_edges: Vec<petgraph::graph::EdgeIndex>,
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
            highlighted_candidates: Vec::new(),
            highlighted_edges: Vec::new(),
        }
    }

    pub fn restart_algoritm(&mut self) {
        self.algo_step = FluerryStep::NotStarted
    }

    pub fn step_algorithm(
        &mut self,
        graph: &mut StableGraph<Circle, Arrow>,
    ) -> Option<CanvasMessage> {
        let cur_state = self.algo_step;

        match self.algo_step {
            FluerryStep::NotStarted => self.initialize_algorithm(graph),
            FluerryStep::Initializing => self.find_start_node(graph),
            FluerryStep::CheckingOutgoing => self.check_outgoing_edges(graph),
            FluerryStep::ChoosingNext => self.choose_next_edge(graph),
            FluerryStep::Advancing => self.advance_to_next(graph),
            FluerryStep::Backtracking => self.backtrack_all(graph),
            FluerryStep::Completed => {
                // Algorithm finished, do nothing
                self.step_explanation = "Algorithm completed".to_string();
                return Some(CanvasMessage::AlgorithmFinished(true));
            }
            FluerryStep::Failed => {
                // Algorithm finished, do nothing
                self.step_explanation = "Algorithm completed".to_string();
                return Some(CanvasMessage::AlgorithmFinished(false));
            }
        }

        //self.update_highlights();
        println!(
            "Step: {} - {}",
            format!("{:?}", cur_state),
            self.step_explanation
        );

        return None;
    }

    fn initialize_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.reset_algorithm(graph);
        self.step_explanation =
            "Initializing algorithm - cloned graph and cleared state".to_string();
    }

    fn find_start_node(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Find a node with outgoing edges (for Eulerian path)
        // For Eulerian circuit, we can start anywhere with edges
        self.current_node = self.graph_clone.node_indices().find(|&node| {
            self.graph_clone
                .edges_directed(node, Direction::Outgoing)
                .count()
                > 0
        });

        if let Some(start_node) = self.current_node {
            self.stack.push(start_node);

            // Highlight starting node with special color
            if let Some(node_weight) = graph.node_weight_mut(start_node) {
                node_weight.highlight_start();
            }

            self.algo_step = FluerryStep::CheckingOutgoing;
            self.step_explanation = format!("Starting from node {:?}", start_node);
        } else {
            self.algo_step = FluerryStep::Completed;
            self.step_explanation = "No suitable start node found - graph may be empty".to_string();
        }
    }

    fn check_outgoing_edges(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Clear temporary highlights from previous step
        self.clear_temporary_highlights(graph);

        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_step = FluerryStep::Failed;
                self.step_explanation = "No current node - algorithm failed".to_string();
                return;
            }
        };

        // Highlight current node being explored
        graph
            .node_weight_mut(current)
            .unwrap()
            .highlight_exploring();

        // Get current outgoing edges with their edge IDs
        self.current_outgoing = self
            .graph_clone
            .edges_directed(current, Direction::Outgoing)
            .map(|edge| (edge.target(), edge.id()))
            .collect();

        if self.current_outgoing.is_empty() {
            // No outgoing edges - need to backtrack and add to circuit
            self.algo_step = FluerryStep::Backtracking;
            self.step_explanation = format!(
                "No outgoing edges from {:?} - backtracking entire stack to complete circuit",
                current
            );
        } else if self.current_outgoing.len() == 1 {
            // Only one choice - take it (even if it's a bridge, we have no choice)
            let (next_node, edge_id) = self.current_outgoing[0];
            self.next_candidate = Some(next_node);

            // Highlight the single edge and target
            graph.edge_weight_mut(edge_id).unwrap().highlight_selected();
            graph.node_weight_mut(next_node).unwrap().highlight_next();
            self.highlighted_edges.push(edge_id);
            self.highlighted_candidates.push(next_node);

            self.algo_step = FluerryStep::Advancing;
            self.step_explanation = format!(
                "Only one outgoing edge from {:?} to {:?} - taking it",
                current, next_node
            );
        } else {
            // Multiple choices - need to choose non-bridge if possible
            self.current_idx = 0;
            self.algo_step = FluerryStep::ChoosingNext;
            self.step_explanation = format!(
                "Multiple outgoing edges ({}) from {:?} - checking for non-bridge",
                self.current_outgoing.len(),
                current
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

        // Clear previous candidate highlights if this isn't the first check
        if self.current_idx > 0 {
            if let Some(&prev_node) = self.highlighted_candidates.last() {
                graph.node_weight_mut(prev_node).unwrap().reset_highlight();
            }
            if let Some(&prev_edge) = self.highlighted_edges.last() {
                if !self.visited_edges.contains(&prev_edge) {
                    graph.edge_weight_mut(prev_edge).unwrap().reset_highlight();
                }
            }
            self.highlighted_candidates.clear();
            self.highlighted_edges.clear();
        }

        if self.current_idx >= self.current_outgoing.len() {
            // Checked all edges, no non-bridge found - use first edge
            // (this means all remaining edges are bridges, which is expected at the end)
            let (next_node, edge_id) = self.current_outgoing[0];
            self.next_candidate = Some(next_node);

            // Highlight the chosen edge
            graph.edge_weight_mut(edge_id).unwrap().highlight_selected();
            graph.node_weight_mut(next_node).unwrap().highlight_next();

            self.algo_step = FluerryStep::Advancing;
            self.step_explanation = format!(
                "All {} edges are bridges - using first available to {:?}",
                self.current_outgoing.len(),
                next_node
            );
            return;
        }

        let (candidate_node, edge_id) = self.current_outgoing[self.current_idx];

        // Highlight current candidate for evaluation
        graph
            .node_weight_mut(candidate_node)
            .unwrap()
            .highlight_candidate();
        self.highlighted_candidates.push(candidate_node);
        self.highlighted_edges.push(edge_id);

        let is_bridge = self.is_bridge(current, candidate_node, edge_id);

        if is_bridge {
            // Mark as bridge and skip to next candidate
            graph.edge_weight_mut(edge_id).unwrap().highlight_bridge();
            self.step_explanation = format!(
                "Edge {:?} to {:?} IS a bridge - skipping ({}/{})",
                current,
                candidate_node,
                self.current_idx + 1,
                self.current_outgoing.len()
            );
            self.current_idx += 1;
            // Stay in ChoosingNext state to check next edge in next step
        } else {
            // Found non-bridge, highlight as selected
            graph.edge_weight_mut(edge_id).unwrap().highlight_selected();
            graph
                .node_weight_mut(candidate_node)
                .unwrap()
                .highlight_next();
            self.next_candidate = Some(candidate_node);
            self.algo_step = FluerryStep::Advancing;
            self.step_explanation = format!(
                "Edge {:?} to {:?} is NOT a bridge - using it ({}/{})",
                current,
                candidate_node,
                self.current_idx + 1,
                self.current_outgoing.len()
            );
        }
    }

    fn advance_to_next(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Clear temporary highlights from evaluation
        self.clear_temporary_highlights(graph);

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

        // Find and remove the edge from working graph
        if let Some(edge_id) = self.graph_clone.find_edge(current, next) {
            self.graph_clone.remove_edge(edge_id);
            self.visited_edges.push(edge_id);

            // Mark edge as part of the path (permanent highlight)
            if let Some(edge_weight) = graph.edge_weight_mut(edge_id) {
                edge_weight.highlight_path();
            }
        }

        // Mark current node as visited in path
        if let Some(node_weight) = graph.node_weight_mut(current) {
            node_weight.highlight_visited();
        }

        // Push current node to stack and move to next node
        self.stack.push(next);
        self.current_node = Some(next);

        self.current_outgoing.clear();
        self.next_candidate = None;
        self.algo_step = FluerryStep::CheckingOutgoing;

        self.step_explanation = format!("Moved from {:?} to {:?}, pushed to stack", current, next);
    }

    fn backtrack_all(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Clear any temporary highlights before backtracking
        self.clear_temporary_highlights(graph);

        let backtrack_count = self.stack.len();

        if backtrack_count == 0 {
            self.algo_step = FluerryStep::Failed;
            self.step_explanation = "Cannot backtrack - stack is empty".to_string();
            return;
        }

        // Process entire stack in one step
        while let Some(node) = self.stack.pop() {
            // Add node to circuit (this builds the Eulerian path in reverse)
            self.circuit.push(node);

            // Highlight the node as part of the final solution
            if let Some(node_weight) = graph.node_weight_mut(node) {
                node_weight.highlight_solution();
            }
        }

        // Check if we're done
        if self.graph_clone.edge_count() == 0 {
            self.algo_step = FluerryStep::Completed;
            self.step_explanation = format!(
                "Backtracked {} nodes and completed circuit with {} total nodes - Eulerian circuit found",
                backtrack_count,
                self.circuit.len()
            );
        } else {
            self.algo_step = FluerryStep::Failed;
            self.step_explanation = format!(
                "Backtracked {} nodes but {} unused edges remain - algorithm failed",
                backtrack_count,
                self.graph_clone.edge_count()
            );
        }

        self.current_node = None;
    }

    fn clear_temporary_highlights(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Clear candidate node highlights
        for &node in &self.highlighted_candidates {
            if let Some(node_weight) = graph.node_weight_mut(node) {
                // Only reset if not part of the solution
                if !self.circuit.contains(&node) && !self.stack.contains(&node) {
                    node_weight.reset_highlight();
                }
            }
        }
        self.highlighted_candidates.clear();

        // Clear temporary edge highlights (but keep solution edges)
        for &edge_id in &self.highlighted_edges {
            if !self.visited_edges.contains(&edge_id) {
                if let Some(edge_weight) = graph.edge_weight_mut(edge_id) {
                    edge_weight.reset_highlight();
                }
            }
        }
        self.highlighted_edges.clear();
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
        self.highlighted_candidates.clear();
        self.highlighted_edges.clear();
        self.clear_highlights(graph);
    }

    // FIXED bridge detection: Count reachable vertices in the entire connected component
    // An edge is a bridge if removing it increases the number of connected components
    fn is_bridge(
        &self,
        from: NodeIndex,
        to: NodeIndex,
        edge_id: petgraph::graph::EdgeIndex,
    ) -> bool {
        // Count connected components before removing the edge
        let components_before = self.count_connected_components(&self.graph_clone);

        // Clone the graph and remove the edge
        let mut temp_graph = self.graph_clone.clone();
        temp_graph.remove_edge(edge_id);

        // Count connected components after removing the edge
        let components_after = self.count_connected_components(&temp_graph);

        // If number of components increased, the edge is a bridge
        let is_bridge = components_after > components_before;

        println!(
            "Bridge check: {:?} -> {:?}: components before={}, after={}, is_bridge={}",
            from, to, components_before, components_after, is_bridge
        );

        is_bridge
    }

    // Count number of connected components in the graph using BFS
    fn count_connected_components(&self, graph: &StableGraph<Circle, Arrow>) -> usize {
        let mut visited = HashSet::new();
        let mut component_count = 0;

        for node in graph.node_indices() {
            if !visited.contains(&node) {
                component_count += 1;
                self.bfs_traversal(graph, node, &mut visited);
            }
        }

        component_count
    }

    // BFS traversal to mark all connected nodes
    fn bfs_traversal(
        &self,
        graph: &StableGraph<Circle, Arrow>,
        start: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
    ) {
        let mut queue = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);

        while let Some(node) = queue.pop_front() {
            // Check all outgoing neighbors
            for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
            // Check all incoming neighbors (for undirected connectivity)
            for neighbor in graph.neighbors_directed(node, Direction::Incoming) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
}
