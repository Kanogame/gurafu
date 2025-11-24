use petgraph::{Direction, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};

use crate::gurafu_app::canvas::{drawable::{link::Link, node::Node}, graph_algorithm::{AlgorithmMessage, GraphAlgorithm}
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
    graph_clone: StableGraph<Node, Link>, // рабочая копия графа (в ней удаляются рёбра)

    // Визуализационные поля
    current_outgoing: Vec<(NodeIndex, petgraph::graph::EdgeIndex)>, // (цель, edge_id)
    current_idx: usize,
    next_candidate: Option<NodeIndex>,
    next_edge: Option<petgraph::graph::EdgeIndex>, // выбранное конкретное ребро (для параллельных рёбер)
    visited_edges: Vec<petgraph::graph::EdgeIndex>, // использованные рёбра (для подсветки)

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
        graph: &mut StableGraph<Node, Link>,
    ) -> Option<AlgorithmMessage> {

        // --------------------------------------------------------
        // ADDED: подсветка текущей вершины на каждом шаге (если она задана)
        // вызываем node_highlight_current() у весa вершины
        // --------------------------------------------------------
        if let Some(cn) = self.current_node {
            if let Some(nw) = graph.node_weight_mut(cn) {
                nw.highlight_start(); // ADDED
            }
        }
        // --------------------------------------------------------

        match self.algo_step {
            HierholzerStep::NotStarted => self.initialize_algorithm(graph),
            HierholzerStep::Initializing => self.find_start_node(graph),
            HierholzerStep::CheckingOutgoing => self.check_outgoing_edges(graph),
            HierholzerStep::ChoosingNext => self.choose_next_edge(graph),
            HierholzerStep::Advancing => self.advance_to_next(graph),
            HierholzerStep::Backtracking => self.backtrack_all(graph),
            HierholzerStep::Completed => {
                let mes = format!(
                        "Алгоритм выполнен успешно, Эйлеров цикл:\n {}",
                        self.circuit
                            .iter()
                            .map(|el| el.index().to_string())
                            .collect::<Vec<String>>()
                            .join(" -> ")
                    );
                
                self.reset_algorithm(graph);

                // На выходе возвращаем circuit в прямом порядке (0->1->2->...)
                return Some(AlgorithmMessage::AlgorithmSuccess(mes));
            }
            HierholzerStep::Failed => {
                for n in graph.node_weights_mut() {
                        n.highlight_error();
                }
                self.reset_algorithm(graph);
                return Some(AlgorithmMessage::AlgorithmFail("Алгоритм завершился, Эйлеров цикл не найден".to_string()));
            }
        }

        self.highlight_current_node(graph);
        None
    }

    fn reset_algorithm(&mut self, graph: &mut StableGraph<Node, Link>) {
        self.graph_clone = graph.clone();
        self.algo_step = HierholzerStep::NotStarted;
        self.stack.clear();
        self.circuit.clear();
        self.current_node = None;
        self.current_outgoing.clear();
        self.current_idx = 0;
        self.next_candidate = None;
        self.next_edge = None;
        self.visited_edges.clear();
        self.highlighted_candidates.clear();
        self.highlighted_edges.clear();

        // сохраняем исходное количество рёбер для финальной валидации
        self.original_edge_count = graph.edge_count();
    }

    fn clear_highlights(&mut self, graph: &mut StableGraph<Node, Link>) {
        for node in graph.node_weights_mut() {
            node.reset_highlight();
        }
        for edge in graph.edge_weights_mut() {
            edge.reset_highlight();
        }
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
            highlighted_candidates: Vec::new(),
            highlighted_edges: Vec::new(),
            original_edge_count: 0,
        }
    }
}

impl HierholzerState {
    /// Инициализация: клонируем граф, считаем исходное число рёбер, очищаем состояние
    fn initialize_algorithm(&mut self, graph: &mut StableGraph<Node, Link>) {
        self.reset_algorithm(graph);

        // Проверка Эйлеровости по степеням (для направленного графа)
        // Если граф не удовлетворяет in_degree == out_degree для какой-либо вершины,
        // то сразу помечаем как Failed (требование для наличия Эйлерова цикла).
        for v in self.graph_clone.node_indices() {
            let out = self.graph_clone.edges_directed(v, Direction::Outgoing).count();
            let inc = self.graph_clone.edges_directed(v, Direction::Incoming).count();

            if out != inc {
                self.algo_step = HierholzerStep::Failed;
                return;
            }
        }
        self.algo_step = HierholzerStep::Initializing
    }

    fn highlight_current_node(&mut self, graph: &mut StableGraph<Node, Link>) {
        if let Some(cur) = self.current_node {
            if let Some(nw) = graph.node_weight_mut(cur) {
                nw.highlight_start(); // ЕДИНЫЙ источник подсветки текущего узла
            }
        }
    }

    /// Находим стартовую вершину (любую с исходящими рёбрами)
    fn find_start_node(&mut self, graph: &mut StableGraph<Node, Link>) {
        self.current_node = self.graph_clone.node_indices().find(|&node| {
            self.graph_clone
                .edges_directed(node, Direction::Outgoing)
                .count()
                > 0
        });

        if let Some(start_node) = self.current_node {
            // начинаем стек с этой вершины
            self.stack.push(start_node);

            // визуальный маркёр старта
            if let Some(node_weight) = graph.node_weight_mut(start_node) {
                node_weight.highlight_start();
            }

            self.algo_step = HierholzerStep::CheckingOutgoing;
        } else {
            // нет рёбер в графе — тривиально завершены (пустой цикл)
            self.algo_step = HierholzerStep::Completed;
        }
    }

    /// Проверяем исходящие рёбра у current_node, готовим варианты
    fn check_outgoing_edges(&mut self, graph: &mut StableGraph<Node, Link>) {
        // очистка временных подсветок перед новым шагом
        self.clear_temporary_highlights(graph);

        let current = match self.current_node {
            Some(n) => n,
            None => {
                self.algo_step = HierholzerStep::Failed;
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
            return;
        } else {
            // Несколько вариантов — для детерминированности выбираем первый (можно изменить UX)
            self.current_idx = 0;
            self.algo_step = HierholzerStep::ChoosingNext;
            return;
        }
    }

    /// Выбор следующего ребра (у нас детерминированный — первый)
    fn choose_next_edge(&mut self, graph: &mut StableGraph<Node, Link>) {
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
                return;
            } else {
                self.algo_step = HierholzerStep::Failed;
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
    }

    /// Выполнить переход по выбранному ребру: удалить ребро в clone, подсветить путь, двигаться дальше
    fn advance_to_next(&mut self, graph: &mut StableGraph<Node, Link>) {
        // очистить временные подсветки
        self.clear_temporary_highlights(graph);

        let current = match self.current_node {
            Some(n) => n,
            None => {
                self.algo_step = HierholzerStep::Failed;
                return;
            }
        };

        graph.node_weight_mut(current).unwrap().highlight_start();

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
    }

    /// Backtracking: в одном шаге снимаем вершины со стека (пока не дойдём до вершины, у которой есть исходящие рёбра),
    /// заполняем circuit. После этого:
    ///  - если рабочий граф пуст — завершаем и формируем итог (реверсим circuit в прямой порядок);
    ///  - если стек не пуст — продолжаем traversal из вершины на вершине стека;
    ///  - если стек пуст, но рёбра остались — ищем в circuit вершину, из которой можно продолжить (merge/rotate);
    ///  - иначе — ошибка.
    fn backtrack_all(&mut self, graph: &mut StableGraph<Node, Link>) {
        // очистка временных подсветок
        self.clear_temporary_highlights(graph);

        // если стек пуст — неоткуда бэктрекать
        if self.stack.is_empty() {
            self.algo_step = HierholzerStep::Failed;
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
        while let Some(top) = self.stack.pop() {
            self.circuit.push(top);
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
                return;
            } else {
                // защита, на случай непредвиденной ситуации
                self.algo_step = HierholzerStep::Failed;
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
                return;
            }

            self.algo_step = HierholzerStep::Completed;
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
                return;
            } else {
                self.algo_step = HierholzerStep::Failed;
                return;
            }
        } else {
            // Рёбра остались, но в circuit нет вершины, из которой можно их достать -> ошибка
            self.algo_step = HierholzerStep::Failed;
            return;
        }
    }

    /// Очистка временных подсветок кандидатов/ребер (но не убираем подсветки пройденного пути)
    fn clear_temporary_highlights(&mut self, graph: &mut StableGraph<Node, Link>) {
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
}
