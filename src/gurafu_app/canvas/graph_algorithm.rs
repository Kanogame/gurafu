use petgraph::prelude::StableGraph;

use crate::gurafu_app::canvas::drawable::{arrow::Arrow, circle::Circle};

pub enum AlgorithmMessage {
    AlgorithmSuccess(String),
    AlgorithmFail(String),
}

pub trait GraphAlgorithm {
    fn new() -> Self;

    fn step_algorithm(
        &mut self,
        graph: &mut StableGraph<Circle, Arrow>,
    ) -> Option<AlgorithmMessage>;

    fn reset_algorithm(&mut self, graph: &mut StableGraph<Circle, Arrow>);
}

impl AlgorithmMessage {
    pub fn get_header(&self) -> String {
        match self {
            AlgorithmMessage::AlgorithmSuccess(_) => "Алгоритм завершен успешно",
            AlgorithmMessage::AlgorithmFail(_) => "Алгоритм завершился с ошибкой",
        }.to_string()
    }

    pub fn get_text(&self) -> &String {
        match self {
            AlgorithmMessage::AlgorithmSuccess(res) => res,
            AlgorithmMessage::AlgorithmFail(res) => res,
        }
    }
}