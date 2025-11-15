use petgraph::{Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};
use std::collections::{HashSet, VecDeque};

use crate::gurafu_app::canvas::{
    AlgorithmMessage,
    drawable::{arrow::Arrow, circle::Circle},
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
                return Some(AlgorithmMessage::AlgorithmSuccess(
                    self.circuit.iter().map(|el| el.index()).collect(),
                ));
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

    fn backtrack_all(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Собираем circuit в правильном порядке из стека
        // Просто копируем стек, так как он уже содержит правильный порядок обхода
        self.circuit = self.stack.clone();

        // Добавляем начальную вершину в конец для замыкания цикла
        if let Some(&start) = self.circuit.first() {
            self.circuit.push(start);
        }

        // Подсвечиваем вершины решения
        for &v in &self.circuit {
            if let Some(n) = graph.node_weight_mut(v) {
                n.highlight_solution();
            }
        }

        let total_edges = self.visited_edges.len();
        let start_end_match = self.circuit.first() == self.circuit.last();
        let expected_circuit_length = total_edges + 1;

        // Более гибкая проверка завершения
        let edges_remaining = self.graph_clone.edge_count();

        if edges_remaining == 0 && start_end_match {
            self.algo_step = FlueryStep::Completed;
            self.step_explanation = format!("Eulerian cycle found with {} edges", total_edges);
        } else {
            self.algo_step = FlueryStep::Failed;
            self.step_explanation = format!(
                "Incomplete circuit: edges_remaining={}, start/end match: {}, circuit length: {} (expected: {})",
                edges_remaining,
                start_end_match,
                self.circuit.len(),
                expected_circuit_length
            );
        }
    }

    fn advance_to_next(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let current = self.current_node.unwrap();
        let next = match self.next_candidate {
            Some(node) => node,
            None => {
                self.algo_step = FlueryStep::Backtracking;
                return;
            }
        };

        // Находим и удаляем ребро из клона графа
        if let Some(edge_id) = self.graph_clone.find_edge(current, next) {
            self.graph_clone.remove_edge(edge_id);
            self.visited_edges.push(edge_id);

            // Подсвечиваем ребро как посещенное
            if let Some(edge) = graph.edge_weight_mut(edge_id) {
                edge.highlight_path();
            }
        }

        // Обновляем подсветку вершин
        if let Some(node) = graph.node_weight_mut(current) {
            node.highlight_visited();
        }

        // Переходим к следующей вершине
        self.current_node = Some(next);

        // Добавляем следующую вершину в стек
        self.stack.push(next);

        self.next_candidate = None;
        self.current_outgoing.clear();
        self.algo_step = FlueryStep::CheckingOutgoing;
    }

    fn choose_next_edge(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        let current = self.current_node.unwrap();

        // Перебираем все возможные ребра, начиная с current_idx
        for i in self.current_idx..self.current_outgoing.len() {
            let (candidate, edge_id) = self.current_outgoing[i];
            let is_bridge = self.is_bridge(current, candidate, edge_id);

            if !is_bridge {
                // Нашли не-мостовое ребро - используем его
                self.next_candidate = Some(candidate);
                self.current_idx = i + 1; // Увеличиваем индекс для следующей итерации

                // Подсвечиваем выбранное ребро и вершину
                if let Some(edge) = graph.edge_weight_mut(edge_id) {
                    edge.highlight_selected();
                }
                if let Some(node) = graph.node_weight_mut(candidate) {
                    node.highlight_next();
                }

                self.algo_step = FlueryStep::Advancing;
                return;
            }
        }

        // Если все ребра - мосты, берем первое доступное
        if let Some(&(first_candidate, first_edge)) = self.current_outgoing.first() {
            self.next_candidate = Some(first_candidate);
            self.current_idx = 0; // Сбрасываем индекс

            if let Some(edge) = graph.edge_weight_mut(first_edge) {
                edge.highlight_selected();
            }
            if let Some(node) = graph.node_weight_mut(first_candidate) {
                node.highlight_next();
            }

            self.algo_step = FlueryStep::Advancing;
        } else {
            self.algo_step = FlueryStep::Backtracking;
        }
    }

    fn is_bridge(&self, from: NodeIndex, to: NodeIndex, eid: petgraph::graph::EdgeIndex) -> bool {
        let mut temp = self.graph_clone.clone();
        temp.remove_edge(eid);

        // Проверяем достижимость to из from в ориентированном графе
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);
        visited.insert(from);

        while let Some(node) = queue.pop_front() {
            for neighbor in temp.neighbors_directed(node, Direction::Outgoing) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        // Если to недостижим из from после удаления ребра - это мост
        !visited.contains(&to)
    }

    fn check_outgoing_edges(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.clear_temporary_highlights(graph);

        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_step = FlueryStep::Failed;
                return;
            }
        };

        // Получаем все исходящие ребра
        let outgoing_edges: Vec<_> = self
            .graph_clone
            .edges_directed(current, Direction::Outgoing)
            .map(|edge| (edge.target(), edge.id()))
            .collect();

        self.current_outgoing = outgoing_edges;
        self.current_idx = 0; // Сбрасываем индекс при каждой новой проверке

        if self.current_outgoing.is_empty() {
            // Нет исходящих ребер - возврат
            self.algo_step = FlueryStep::Backtracking;
            return;
        }

        // Переходим к выбору следующего ребра
        self.algo_step = FlueryStep::ChoosingNext;
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
                .count()
                > 0
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
