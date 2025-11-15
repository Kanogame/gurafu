use petgraph::{Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};
use std::collections::{HashSet, VecDeque};

use crate::gurafu_app::canvas::{drawable::{arrow::Arrow, circle::Circle}, graph_algorithm::{AlgorithmMessage, AlgorithmResultDisplay, GraphAlgorithm}
};

mod test_hierholzer;

/// Состояние алгоритма Hierholzer -- сохранено имя структуры для совместимости с UI
#[derive(Clone)]
pub struct HierholzerState {
    // Алгоритмическое состояние
    algo_step: HierholzerStep,
    stack: Vec<NodeIndex>,          // стек текущего пути
    circuit: Vec<NodeIndex>,        // итоговый цикл (будет приведён в прямой порядок при завершении)
    current_node: Option<NodeIndex>,// текущая вершина
    graph_clone: StableGraph<Circle, Arrow>, // рабочая копия графа (в ней удаляются рёбра)

    // Визуализационные поля
    current_outgoing: Vec<(NodeIndex, petgraph::graph::EdgeIndex)>, // (цель, edge_id)
    current_idx: usize,
    next_candidate: Option<NodeIndex>,
    next_edge: Option<petgraph::graph::EdgeIndex>, // выбранное конкретное ребро (для параллельных рёбер)
    visited_edges: Vec<petgraph::graph::EdgeIndex>, // использованные рёбра (для подсветки)
    step_explanation: String,

    // Для очистки временных подсветок
    highlighted_candidates: Vec<NodeIndex>,
    highlighted_edges: Vec<petgraph::graph::EdgeIndex>,

    // Количество рёбер в исходном графе (нужно для проверки корректности)
    original_edge_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum HierholzerStep {
    NotStarted,
    Initializing,
    CheckingOutgoing,
    ChoosingNext,
    Advancing,
    Backtracking,
    Completed,
    Failed,
}

impl GraphAlgorithm for HierholzerState {
    fn step_algorithm(
        &mut self,
        graph: &mut StableGraph<Circle, Arrow>,
    ) -> Option<AlgorithmMessage> {
        let cur_state = self.algo_step;

        match self.algo_step {
            HierholzerStep::NotStarted => self.initialize_algorithm(graph),
            HierholzerStep::Initializing => self.find_start_node(graph),
            HierholzerStep::CheckingOutgoing => self.check_outgoing_edges(graph),
            HierholzerStep::ChoosingNext => self.choose_next_edge(graph),
            HierholzerStep::Advancing => self.advance_to_next(graph),
            HierholzerStep::Backtracking => self.backtrack_all(graph),
            HierholzerStep::Completed => {
                self.step_explanation = "Algorithm completed successfully".into();

                // На выходе возвращаем circuit в прямом порядке (0->1->2->...)
                return Some(AlgorithmMessage::AlgorithmSuccess(
                    format!(
                        "Алгоритм выполнен успешно, Эйлеров цикл: {}",
                        self.circuit
                            .iter()
                            .map(|el| el.index().to_string())
                            .collect::<Vec<String>>()
                            .join(" -> ")
                    )
                ));
            }
            HierholzerStep::Failed => {
                self.step_explanation = "Algorithm failed (graph not Eulerian)".into();
                return Some(AlgorithmMessage::AlgorithmFail("Алгоритм завершился, Эйлеров цикл не найден".to_string()));
            }
        }

        println!(
            "Step: {} - {}",
            format!("{:?}", cur_state),
            self.step_explanation
        );

        None
    }

    fn reset_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.graph_clone = graph.clone();
        self.algo_step = HierholzerStep::Initializing;
        self.stack.clear();
        self.circuit.clear();
        self.current_node = None;
        self.current_outgoing.clear();
        self.current_idx = 0;
        self.next_candidate = None;
        self.next_edge = None;
        self.visited_edges.clear();
        self.step_explanation.clear();
        self.highlighted_candidates.clear();
        self.highlighted_edges.clear();

        // сохраняем исходное количество рёбер для финальной валидации
        self.original_edge_count = graph.edge_count();

        self.clear_highlights(graph);
    }

    fn new() -> Self {
        Self {
            algo_step: HierholzerStep::NotStarted,
            stack: Vec::new(),
            circuit: Vec::new(),
            current_node: None,
            graph_clone: StableGraph::new(),
            current_outgoing: Vec::new(),
            current_idx: 0,
            next_candidate: None,
            next_edge: None,
            visited_edges: Vec::new(),
            step_explanation: String::new(),
            highlighted_candidates: Vec::new(),
            highlighted_edges: Vec::new(),
            original_edge_count: 0,
        }
    }
}

impl HierholzerState {
    /// Инициализация: клонируем граф, считаем исходное число рёбер, очищаем состояние
    fn initialize_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        self.reset_algorithm(graph);

        // === ДОБАВЛЕНО: Проверка Эйлеровости по степеням (для направленного графа)
        // Если граф не удовлетворяет in_degree == out_degree для какой-либо вершины,
        // то сразу помечаем как Failed (требование для наличия Эйлерова цикла).
        for v in self.graph_clone.node_indices() {
            let out = self.graph_clone.edges_directed(v, Direction::Outgoing).count();
            let inc = self.graph_clone.edges_directed(v, Direction::Incoming).count();

            if out != inc {
                self.algo_step = HierholzerStep::Failed;
                self.step_explanation = format!(
                    "Graph is not Eulerian: at node {:?} out_degree={} != in_degree={}",
                    v, out, inc
                );
                return;
            }
        }
        // ========================================

        self.step_explanation =
            "Initializing Hierholzer - cloned graph and cleared state".to_string();
    }

    /// Находим стартовую вершину (любую с исходящими рёбрами)
    fn find_start_node(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Если graph_clone ещё пустой (reset уже клонировал), ищем вершину с исходящими ребрами
        self.current_node = self.graph_clone.node_indices().find(|&node| {
            self.graph_clone
                .edges_directed(node, Direction::Outgoing)
                .count()
                > 0
        });

        // Защита: если original_edge_count ещё не установлено (например, тесты напрямую подставили graph_clone),
        // установим его здесь по текущему clone.
        if self.original_edge_count == 0 {
            self.original_edge_count = self.graph_clone.edge_count();
        }

        if let Some(start_node) = self.current_node {
            // начинаем стек с этой вершины
            self.stack.push(start_node);

            // визуальный маркёр старта
            if let Some(node_weight) = graph.node_weight_mut(start_node) {
                node_weight.highlight_start();
            }

            self.algo_step = HierholzerStep::CheckingOutgoing;
            self.step_explanation = format!("Starting Hierholzer from node {:?}", start_node);
        } else {
            // нет рёбер в графе — тривиально завершены (пустой цикл)
            self.algo_step = HierholzerStep::Completed;
            self.step_explanation = "Graph has no edges - nothing to traverse".to_string();
        }
    }

    /// Проверяем исходящие рёбра у current_node, готовим варианты
    fn check_outgoing_edges(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // очистка временных подсветок перед новым шагом
        self.clear_temporary_highlights(graph);

        let current = match self.current_node {
            Some(n) => n,
            None => {
                self.algo_step = HierholzerStep::Failed;
                self.step_explanation = "No current node - algorithm failed".to_string();
                return;
            }
        };

        // подсветка исследуемой вершины
        if let Some(nw) = graph.node_weight_mut(current) {
            nw.highlight_exploring();
        }

        // собираем все исходящие рёбра (с конкретными id — важно для параллельных рёбер)
        self.current_outgoing = self
            .graph_clone
            .edges_directed(current, Direction::Outgoing)
            .map(|e| (e.target(), e.id()))
            .collect();

        if self.current_outgoing.is_empty() {
            // Если нет исходящих — пора делать backtrack (закрыть текущий путь)
            self.algo_step = HierholzerStep::Backtracking;
            self.step_explanation = format!("No outgoing edges from {:?} - need to backtrack", current);
            return;
        } else if self.current_outgoing.len() == 1 {
            // Одна опция — выбираем её
            let (next_node, edge_id) = self.current_outgoing[0];
            self.next_candidate = Some(next_node);
            self.next_edge = Some(edge_id);

            if let Some(e) = graph.edge_weight_mut(edge_id) {
                e.highlight_selected();
            }
            if let Some(nw) = graph.node_weight_mut(next_node) {
                nw.highlight_next();
            }
            self.highlighted_edges.push(edge_id);
            self.highlighted_candidates.push(next_node);

            self.algo_step = HierholzerStep::Advancing;
            self.step_explanation = format!("Single outgoing from {:?} -> {:?}, taking it", current, next_node);
            return;
        } else {
            // Несколько вариантов — для детерминированности выбираем первый (можно изменить UX)
            self.current_idx = 0;
            self.algo_step = HierholzerStep::ChoosingNext;
            self.step_explanation = format!("Multiple outgoing edges ({}) from {:?} - selecting first", self.current_outgoing.len(), current);
            return;
        }
    }

    /// Выбор следующего ребра (у нас детерминированный — первый)
    fn choose_next_edge(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Если вдруг индекс выходит за диапазон — используем fallback на первый
        if self.current_idx >= self.current_outgoing.len() {
            if let Some((node, edge_id)) = self.current_outgoing.get(0).cloned() {
                self.next_candidate = Some(node);
                self.next_edge = Some(edge_id);
                if let Some(e) = graph.edge_weight_mut(edge_id) { e.highlight_selected(); }
                if let Some(nw) = graph.node_weight_mut(node) { nw.highlight_next(); }
                self.highlighted_candidates.push(node);
                self.highlighted_edges.push(edge_id);
                self.algo_step = HierholzerStep::Advancing;
                self.step_explanation = "No preferable choice found - using first outgoing".to_string();
                return;
            } else {
                self.algo_step = HierholzerStep::Failed;
                self.step_explanation = "No outgoing edges found during choose_next".to_string();
                return;
            }
        }

        let (candidate_node, edge_id) = self.current_outgoing[self.current_idx];

        // подсветка кандидата
        if let Some(nw) = graph.node_weight_mut(candidate_node) {
            nw.highlight_candidate();
        }
        if let Some(e) = graph.edge_weight_mut(edge_id) {
            e.highlight_selected();
        }
        self.highlighted_candidates.push(candidate_node);
        self.highlighted_edges.push(edge_id);

        // фиксируем выбор
        self.next_candidate = Some(candidate_node);
        self.next_edge = Some(edge_id);
        self.algo_step = HierholzerStep::Advancing;
        self.step_explanation = format!("Selected edge {:?} -> {:?} for traversal", edge_id, candidate_node);
    }

    /// Выполнить переход по выбранному ребру: удалить ребро в clone, подсветить путь, двигаться дальше
    fn advance_to_next(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // очистить временные подсветки
        self.clear_temporary_highlights(graph);

        let current = match self.current_node {
            Some(n) => n,
            None => {
                self.algo_step = HierholzerStep::Failed;
                return;
            }
        };

        let next = match self.next_candidate {
            Some(n) => n,
            None => {
                self.algo_step = HierholzerStep::Failed;
                return;
            }
        };

        // удаляем конкретное ребро (если оно ещё есть) — важная защита для параллельных рёбер
        if let Some(edge_id) = self.next_edge {
            let removed = self.graph_clone.remove_edge(edge_id);
            if removed.is_some() {
                self.visited_edges.push(edge_id);
                if let Some(ew) = graph.edge_weight_mut(edge_id) {
                    ew.highlight_path(); // постоянная подсветка пути
                }
            } else {
                // fallback: удаляем любое ребро current->next, если конкретного id уже нет
                if let Some(real_edge) = self.graph_clone.find_edge(current, next) {
                    self.graph_clone.remove_edge(real_edge);
                    self.visited_edges.push(real_edge);
                    if let Some(ew) = graph.edge_weight_mut(real_edge) {
                        ew.highlight_path();
                    }
                }
            }
        } else {
            // если id не задан — удаляем первое найденное ребро current->next
            if let Some(real_edge) = self.graph_clone.find_edge(current, next) {
                self.graph_clone.remove_edge(real_edge);
                self.visited_edges.push(real_edge);
                if let Some(ew) = graph.edge_weight_mut(real_edge) {
                    ew.highlight_path();
                }
            }
        }

        // отмечаем текущую вершину как посещённую в пути
        if let Some(nw) = graph.node_weight_mut(current) {
            nw.highlight_visited();
        }

        // продвигаемся: пушим текущую вершину в стек и переходим к next
        self.stack.push(next);
        self.current_node = Some(next);

        // сброс временных полей
        self.current_outgoing.clear();
        self.current_idx = 0;
        self.next_candidate = None;
        self.next_edge = None;

        self.algo_step = HierholzerStep::CheckingOutgoing;
        self.step_explanation = format!("Advanced from {:?} to {:?}, pushed to stack", current, next);
    }

    /// Backtracking: в одном шаге снимаем вершины со стека (пока не дойдём до вершины, у которой есть исходящие рёбра),
    /// заполняем circuit. После этого:
    ///  - если рабочий граф пуст — завершаем и формируем итог (реверсим circuit в прямой порядок);
    ///  - если стек не пуст — продолжаем traversal из вершины на вершине стека;
    ///  - если стек пуст, но рёбра остались — ищем в circuit вершину, из которой можно продолжить (merge/rotate);
    ///  - иначе — ошибка.
    fn backtrack_all(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // очистка временных подсветок
        self.clear_temporary_highlights(graph);

        // если стек пуст — неоткуда бэктрекать
        if self.stack.is_empty() {
            self.algo_step = HierholzerStep::Failed;
            self.step_explanation = "Backtrack requested but stack is empty".to_string();
            return;
        }

        // Попарно снимаем вершины со стека и добавляем в circuit до тех пор,
        // пока верх стека НЕ имеет исходящих рёбер (в рабочем graph_clone),
        // либо пока стек не опустеет.
        //
        // Это корректно: мы снимаем именно те вершины, у которых уже не осталось
        // ни одного необработанного исходящего ребра (локальный walk закрыт).
        //
        // Важно: делаем это в одном шаге (для компактной визуализации).
        let mut popped = 0usize;
        while let Some(top) = self.stack.pop() {
            self.circuit.push(top);
            popped += 1;
            if let Some(nw) = graph.node_weight_mut(top) {
                nw.highlight_solution();
            }

            // Если стек пуст — выйдем и обработаем случай ниже
            if self.stack.is_empty() {
                break;
            }

            // Проверим новый верх стека: если у него ещё есть исходящие рёбра, то остановимся —
            // мы сможем продолжить traversal из этой вершины (через обычный цикл step'ов).
            if let Some(&new_top) = self.stack.last() {
                let out_count = self.graph_clone.edges_directed(new_top, Direction::Outgoing).count();
                if out_count > 0 {
                    // Оставляем новый top в стеке (мы его не снимали) — нужно продолжать из него.
                    break;
                } else {
                    // иначе продолжаем снимать следующий элемент
                    continue;
                }
            } else {
                break;
            }
        }

        // Если после снятия стек не пуст — продолжаем traversal из вершины на вершине стека.
        if !self.stack.is_empty() {
            if let Some(&resume_v) = self.stack.last() {
                self.current_node = Some(resume_v);
                self.algo_step = HierholzerStep::CheckingOutgoing;
                self.step_explanation = format!(
                    "Backtracked {} nodes; resuming traversal from {:?} (stack non-empty).",
                    popped,
                    resume_v
                );
                return;
            } else {
                // защита, на случай непредвиденной ситуации
                self.algo_step = HierholzerStep::Failed;
                self.step_explanation = "Backtracked but cannot find resume vertex on stack".to_string();
                return;
            }
        }

        // Если стек пуст — смотрим на оставшиеся рёбра.
        // Если рёбер нет — успешно завершили; иначе пытаемся найти в circuit вершину, из которой можно продолжить.
        if self.graph_clone.edge_count() == 0 {
            // circuit сейчас построен в порядке обратном обходу (это стандарт Hierholzer).
            // Нам нужен прямой порядок (0->1->2->...), поэтому реверсим.
            self.circuit.reverse();

            // Вычислим ожидаемое исходное число рёбер надежно:
            // если original_edge_count установлен — используем его, иначе — полагаться на visited_edges.len().
            let computed_original = if self.original_edge_count == 0 {
                self.visited_edges.len()
            } else {
                self.original_edge_count
            };

            if self.visited_edges.len() != computed_original {
                self.algo_step = HierholzerStep::Failed;
                self.step_explanation = format!(
                    "Completed backtrack but used edges {} != expected {} => Failed",
                    self.visited_edges.len(),
                    computed_original
                );
                return;
            }

            self.algo_step = HierholzerStep::Completed;
            self.step_explanation = format!(
                "Completed Eulerian circuit - popped {} nodes; circuit length {}",
                popped,
                self.circuit.len()
            );
            self.current_node = None;
            return;
        }

        // Рёбра остались и стек пуст — нужно найти в circuit вершину, у которой есть исходящие рёбра
        // (мердж циклов): вращаем circuit, чтобы эта вершина стала последней, и продолжаем traversal.
        if let Some(pos) = self.circuit.iter().position(|&v| {
            self.graph_clone.edges_directed(v, Direction::Outgoing).count() > 0
        }) {
            let n = self.circuit.len();
            let mut rotated: Vec<NodeIndex> = Vec::with_capacity(n);
            // add pos+1 .. n-1
            for i in (pos + 1)..n {
                rotated.push(self.circuit[i]);
            }
            // add 0 ..= pos
            for i in 0..=pos {
                rotated.push(self.circuit[i]);
            }
            self.circuit = rotated;

            // resume from last element in rotated circuit
            if let Some(&start_next) = self.circuit.last() {
                self.current_node = Some(start_next);
                self.stack.clear();
                self.stack.push(start_next);

                if let Some(nw) = graph.node_weight_mut(start_next) {
                    nw.highlight_exploring();
                }

                self.algo_step = HierholzerStep::CheckingOutgoing;
                self.step_explanation = format!(
                    "Merged cycles: found circuit vertex {:?} with remaining edges; resumed traversal",
                    start_next
                );
                return;
            } else {
                self.algo_step = HierholzerStep::Failed;
                self.step_explanation = "Rotation produced empty circuit unexpectedly".to_string();
                return;
            }
        } else {
            // Рёбра остались, но в circuit нет вершины, из которой можно их достать -> ошибка
            self.algo_step = HierholzerStep::Failed;
            self.step_explanation = format!(
                "Backtracked {} nodes but {} edges remain and no circuit vertex can reach them - failed",
                popped,
                self.graph_clone.edge_count()
            );
            return;
        }
    }

    /// Очистка временных подсветок кандидатов/ребер (но не убираем подсветки пройденного пути)
    fn clear_temporary_highlights(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        // Очистка подсветки вершин-кандидатов
        for &node in &self.highlighted_candidates {
            if let Some(node_weight) = graph.node_weight_mut(node) {
                // сбрасываем подсветку только если вершина не в circuit и не в стеке (т.е. не часть решения)
                if !self.circuit.contains(&node) && !self.stack.contains(&node) {
                    node_weight.reset_highlight();
                }
            }
        }
        self.highlighted_candidates.clear();

        // Очистка временных подсветок рёбер (если ребро не в visited_edges)
        for &edge_id in &self.highlighted_edges {
            if !self.visited_edges.contains(&edge_id) {
                if let Some(edge_weight) = graph.edge_weight_mut(edge_id) {
                    edge_weight.reset_highlight();
                }
            }
        }
        self.highlighted_edges.clear();
    }

    /// Вспомогательная очистка кандидатских подсветок (между проверками)
    fn clear_candidate_highlights(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        for &node in &self.highlighted_candidates {
            if let Some(node_weight) = graph.node_weight_mut(node) {
                if !self.circuit.contains(&node) && !self.stack.contains(&node) {
                    node_weight.reset_highlight();
                }
            }
        }
        for &edge_id in &self.highlighted_edges {
            if !self.visited_edges.contains(&edge_id) {
                if let Some(edge_weight) = graph.edge_weight_mut(edge_id) {
                    edge_weight.reset_highlight();
                }
            }
        }
        self.highlighted_candidates.clear();
        self.highlighted_edges.clear();
    }

    /// Полная очистка всех подсветок (reset)
    fn clear_highlights(&mut self, graph: &mut StableGraph<Circle, Arrow>) {
        for node in graph.node_weights_mut() {
            node.reset_highlight();
        }
        for edge in graph.edge_weights_mut() {
            edge.reset_highlight();
        }
    }

    
}
