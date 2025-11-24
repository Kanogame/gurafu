use petgraph::prelude::StableGraph;

use crate::gurafu_app::canvas::drawable::{link::Link, node::Node};

pub enum AlgorithmMessage {
    AlgorithmSuccess(String),
    AlgorithmFail(String),
}

pub trait GraphAlgorithm {
    fn new() -> Self;

    fn step_algorithm(
        &mut self,
        graph: &mut StableGraph<Node, Link>,
    ) -> Option<AlgorithmMessage>;

    fn reset_algorithm(&mut self, graph: &mut StableGraph<Node, Link>);
    fn clear_highlights(&mut self, graph: &mut StableGraph<Node, Link>);
}

impl AlgorithmMessage {
    pub fn get_header(&self) -> String {
        match self {
            AlgorithmMessage::AlgorithmSuccess(_) => "Эйлеров цикл найден",
            AlgorithmMessage::AlgorithmFail(_) => "Эйлеров цикл не найден",
        }.to_string()
    }

    pub fn get_text(&self) -> &String {
        match self {
            AlgorithmMessage::AlgorithmSuccess(res) => res,
            AlgorithmMessage::AlgorithmFail(res) => res,
        }
    }
}