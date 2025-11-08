use petgraph::{Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};
use std::collections::{HashSet, VecDeque};

use crate::gurafu_app::canvas::{
    AlgorithmMessage, drawable::{arrow::Arrow, circle::Circle}
};

#[derive(Clone)]
pub struct FlueryState {
    // Algorithm state
    algo_step: FlueryStep,
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

mod test_fluery;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlueryStep {
    NotStarted,
    Initializing,
    CheckingOutgoing,
    ChoosingNext,
    Advancing,
    Backtracking,
    Completed,
    Failed,
}



impl FlueryState {
    pub fn new() -> Self {
        Self {
            algo_step: FlueryStep::NotStarted,
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

    pub fn step_algorithm(
        &mut self,
        graph: &mut StableGraph<Circle, Arrow>,
    ) -> Option<AlgorithmMessage> {
        match self.algo_step {
            FlueryStep::NotStarted => self.initialize_algorithm(graph),
            FlueryStep::Initializing => self.find_start_node(graph),
            FlueryStep::CheckingOutgoing => self.check_outgoing_edges(graph),
            FlueryStep::ChoosingNext => self.choose_next_edge(graph),
            FlueryStep::Advancing => self.advance_to_next(graph),
            FlueryStep::Backtracking => self.backtrack_all(graph),
            FlueryStep::Completed => {
                self.step_explanation = "Algorithm completed successfully".into();
                return Some(AlgorithmMessage::AlgorithmSuccess(self.circuit.iter().map(|el| el.index()).collect()));
            }
            FlueryStep::Failed => {
                self.step_explanation = "Algorithm failed (graph not Eulerian)".into();
                return Some(AlgorithmMessage::AlgorithmFail);
            }
        }
        None
    }

    fn initialize_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.reset_algorithm(graph);
        self.step_explanation = "Initializing algorithm".into();
    }

    fn find_start_node(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
    // === 1️⃣ Проверка эйлеровости графа ===
    if !Self::is_eulerian_directed(&self.graph_clone) {
        self.algo_step = FlueryStep::Failed;
        self.step_explanation =
            "Graph is not Eulerian: not strongly connected or in/out degrees differ".into();
        return;
    }

    // === 2️⃣ Поиск стартовой вершины ===
    self.current_node = self.graph_clone.node_indices().find(|&n| {
        self.graph_clone
            .edges_directed(n, Direction::Outgoing)
            .count() > 0
    });

    if let Some(start) = self.current_node {
        self.stack.push(start);
        if let Some(node) = graph.node_weight_mut(start) {
            node.highlight_start();
        }
        self.algo_step = FlueryStep::CheckingOutgoing;
        self.step_explanation = format!("Starting from node {:?}", start);
    } else {
        self.algo_step = FlueryStep::Failed;
        self.step_explanation = "No start node found".into();
    }
}

fn is_eulerian_directed(graph: &StableGraph<Circle, Arrow>) -> bool {
    // Проверяем: граф должен быть сильно связным
    use petgraph::algo::kosaraju_scc;

    let scc = kosaraju_scc(graph);
    if scc.len() != 1 {
        return false;
    }

    // Проверяем баланс входящих/исходящих рёбер
    for node in graph.node_indices() {
        let in_deg = graph.edges_directed(node, Direction::Incoming).count();
        let out_deg = graph.edges_directed(node, Direction::Outgoing).count();
        if in_deg != out_deg {
            return false;
        }
    }
    true
}



    fn check_outgoing_edges(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.clear_temporary_highlights(graph);

        let cur = match self.current_node {
            Some(v) => v,
            None => {
                self.algo_step = FlueryStep::Failed;
                return;
            }
        };

        let outgoing: Vec<_> = self.graph_clone
            .edges_directed(cur, Direction::Outgoing)
            .map(|e| (e.target(), e.id()))
            .collect();

        self.current_outgoing = outgoing;

        if self.current_outgoing.is_empty() {
            self.algo_step = FlueryStep::Backtracking;
            return;
        }

        if self.current_outgoing.len() == 1 {
            let (next, eid) = self.current_outgoing[0];
            graph.edge_weight_mut(eid).unwrap().highlight_selected();
            graph.node_weight_mut(next).unwrap().highlight_next();
            self.next_candidate = Some(next);
            self.algo_step = FlueryStep::Advancing;
            return;
        }

        self.current_idx = 0;
        self.algo_step = FlueryStep::ChoosingNext;
    }

    fn choose_next_edge(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let cur = self.current_node.unwrap();
        if self.current_idx >= self.current_outgoing.len() {
            let (next, eid) = self.current_outgoing[0];
            self.next_candidate = Some(next);
            graph.edge_weight_mut(eid).unwrap().highlight_selected();
            self.algo_step = FlueryStep::Advancing;
            return;
        }

        let (candidate, eid) = self.current_outgoing[self.current_idx];
        let is_bridge = self.is_bridge(cur, candidate, eid);

        if is_bridge {
            self.current_idx += 1;
            return;
        } else {
            self.next_candidate = Some(candidate);
            graph.edge_weight_mut(eid).unwrap().highlight_selected();
            self.algo_step = FlueryStep::Advancing;
        }
    }

    fn advance_to_next(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let cur = self.current_node.unwrap();
        let next = self.next_candidate.unwrap();

        if let Some(eid) = self.graph_clone.find_edge(cur, next) {
            self.graph_clone.remove_edge(eid);
            self.visited_edges.push(eid);
            if let Some(e) = graph.edge_weight_mut(eid) {
                e.highlight_path();
            }
        }

        if let Some(node) = graph.node_weight_mut(cur) {
            node.highlight_visited();
        }

        self.stack.push(next);
        self.current_node = Some(next);
        self.next_candidate = None;
        self.current_outgoing.clear();
        self.algo_step = FlueryStep::CheckingOutgoing;
    }

    fn backtrack_all(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        while let Some(v) = self.stack.pop() {
            self.circuit.push(v);
            if let Some(n) = graph.node_weight_mut(v) {
                n.highlight_solution();
            }
        }

        let total_edges = self.visited_edges.len();
        let ok = self.graph_clone.edge_count() == 0
            && self.circuit.first() == self.circuit.last()
            && self.circuit.len() == total_edges + 1;

        if ok {
            self.algo_step = FlueryStep::Completed;
            self.step_explanation = format!("Eulerian cycle found with {} edges", total_edges);
        } else {
            self.algo_step = FlueryStep::Failed;
            self.step_explanation = format!(
                "Incomplete circuit: edges_remaining={}, start/end mismatch or skipped edge",
                self.graph_clone.edge_count()
            );
        }
    }

    fn is_bridge(
        &self,
        from: NodeIndex,
        to: NodeIndex,
        eid: petgraph::graph::EdgeIndex,
    ) -> bool {
        let mut temp = self.graph_clone.clone();
        temp.remove_edge(eid);

        // Проверяем, достижим ли 'to' из 'from' в изменённом графе
        let mut visited = HashSet::new();
        let mut q = VecDeque::new();
        q.push_back(from);
        visited.insert(from);

        while let Some(u) = q.pop_front() {
            for nb in temp.neighbors_directed(u, Direction::Outgoing) {
                if visited.insert(nb) {
                    q.push_back(nb);
                }
            }
        }

        !visited.contains(&to)
    }

    fn clear_temporary_highlights(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        for &n in &self.highlighted_candidates {
            if let Some(node) = graph.node_weight_mut(n) {
                node.reset_highlight();
            }
        }
        self.highlighted_candidates.clear();
        for &e in &self.highlighted_edges {
            if let Some(edge) = graph.edge_weight_mut(e) {
                edge.reset_highlight();
            }
        }
        self.highlighted_edges.clear();
    }

    pub fn reset_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.graph_clone = graph.clone();
        self.algo_step = FlueryStep::Initializing;
        self.stack.clear();
        self.circuit.clear();
        self.current_node = None;
        self.current_outgoing.clear();
        self.current_idx = 0;
        self.next_candidate = None;
        self.visited_edges.clear();
        self.highlighted_candidates.clear();
        self.highlighted_edges.clear();
        for n in graph.node_weights_mut() {
            n.reset_highlight();
        }
        for e in graph.edge_weights_mut() {
            e.reset_highlight();
        }
    }
}
