#[cfg(test)]
mod test_fluery {
    use crate::gurafu_app::canvas::drawable::arrow::Arrow;
    use crate::gurafu_app::canvas::drawable::circle::Circle;
    use crate::gurafu_app::canvas::fluery::{FlueryState, FlueryStep};

    use petgraph::prelude::StableGraph;
    use petgraph::stable_graph::NodeIndex;
    use rand::seq::SliceRandom;

    fn add_edge(
        g: &mut StableGraph<Circle, Arrow>,
        from: usize,
        to: usize,
    ) {
        let from_idx = NodeIndex::new(from);
        let to_idx = NodeIndex::new(to);
        g.add_edge(from_idx, to_idx, Arrow::default());
    }

    fn make_circle_graph(n: usize) -> StableGraph<Circle, Arrow> {
        let mut g = StableGraph::<Circle, Arrow>::new();
        for _ in 0..n {
            g.add_node(Circle::default());
        }
        g
    }

    fn run_fluery(mut algo: FlueryState, mut graph: StableGraph<Circle, Arrow>) -> FlueryStep {
        algo.algo_step = FlueryStep::CheckingOutgoing;
        while algo.algo_step != FlueryStep::Completed && algo.algo_step != FlueryStep::Failed {
            algo.step_algorithm(&mut graph);
        }
        algo.algo_step
    }

    /// Примитив 1: петля на одном узле
    #[test]
    fn trivial_self_loop() {
        let mut g = make_circle_graph(1);
        add_edge(&mut g, 0, 0);

        let mut algo = FlueryState::new();
        algo.graph_clone = g.clone();
        algo.current_node = Some(NodeIndex::new(0));
        algo.stack.push(NodeIndex::new(0));

        let result = run_fluery(algo, g);
        assert_eq!(result, FlueryStep::Completed);
    }

    /// Примитив 2: треугольник 0→1→2→0
    #[test]
    fn simple_triangle_cycle() {
        let mut g = make_circle_graph(3);
        add_edge(&mut g, 0, 1);
        add_edge(&mut g, 1, 2);
        add_edge(&mut g, 2, 0);

        let mut algo = FlueryState::new();
        algo.graph_clone = g.clone();
        algo.current_node = Some(NodeIndex::new(0));
        algo.stack.push(NodeIndex::new(0));

        let result = run_fluery(algo, g);
        assert_eq!(result, FlueryStep::Completed);
    }

    /// Контрпример: 0→3→2→1→0 и 0→2
    #[test]
    fn counterexample_false_cycle_should_fail() {
        let mut g = make_circle_graph(4);
        add_edge(&mut g, 0, 3);
        add_edge(&mut g, 3, 2);
        add_edge(&mut g, 2, 1);
        add_edge(&mut g, 1, 0);
        add_edge(&mut g, 0, 2); // лишнее ребро

        let mut algo = FlueryState::new();
        algo.graph_clone = g.clone();
        algo.current_node = Some(NodeIndex::new(0));
        algo.stack.push(NodeIndex::new(0));

        let result = run_fluery(algo, g);
        assert_eq!(result, FlueryStep::Failed, "Алгоритм должен завершиться с ошибкой");
    }

    /// Сложный: два цикла, соединённые мостом
    /// 0→1→2→0 и 0→3→4→0
    #[test]
    fn double_nested_cycle() {
        let mut g = make_circle_graph(5);
        add_edge(&mut g, 0, 1);
        add_edge(&mut g, 1, 2);
        add_edge(&mut g, 2, 0);
        add_edge(&mut g, 0, 3);
        add_edge(&mut g, 3, 4);
        add_edge(&mut g, 4, 0);

        let mut algo = FlueryState::new();
        algo.graph_clone = g.clone();
        algo.current_node = Some(NodeIndex::new(0));
        algo.stack.push(NodeIndex::new(0));

        let result = run_fluery(algo, g);
        assert_eq!(result, FlueryStep::Completed);
    }

    /// Сложный 2: цикл с хвостом
    /// 0→1→2→0 и 2→3
    #[test]
    fn cycle_with_tail_should_fail() {
        let mut g = make_circle_graph(4);
        add_edge(&mut g, 0, 1);
        add_edge(&mut g, 1, 2);
        add_edge(&mut g, 2, 0);
        add_edge(&mut g, 2, 3);

        let mut algo = FlueryState::new();
        algo.graph_clone = g.clone();
        algo.current_node = Some(NodeIndex::new(0));
        algo.stack.push(NodeIndex::new(0));

        let result = run_fluery(algo, g);
        assert_eq!(result, FlueryStep::Failed);
    }

    /// Случайный эйлеров граф: 6 узлов, множество циклов
    #[test]
    fn random_eulerian_graph() {
        let mut g = make_circle_graph(6);
        let mut rng = rand::rng();
        let mut nodes: Vec<usize> = (0..6).collect();

        for _ in 0..3 {
            nodes.shuffle(&mut rng);
            for i in 0..nodes.len() {
                let a = nodes[i];
                let b = nodes[(i + 1) % nodes.len()];
                add_edge(&mut g, a, b);
            }
        }

        let mut algo = FlueryState::new();
        algo.graph_clone = g.clone();
        algo.current_node = Some(NodeIndex::new(0));
        algo.stack.push(NodeIndex::new(0));

        let result = run_fluery(algo, g);
        assert_eq!(result, FlueryStep::Completed);
    }
}
