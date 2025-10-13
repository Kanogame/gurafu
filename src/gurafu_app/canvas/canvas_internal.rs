use iced::{Color, Point, Theme, mouse, widget::canvas};
use petgraph::{Direction, algo, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef};

use crate::gurafu_app::{
    canvas::{
        CanvasMessage,
        camera::Camera,
        canvas_internal::grid::Grid,
        drawable::{arrow::Arrow, circle::Circle},
        helpers,
    },
    toolbar::ToolbarOptions,
};

mod grid;

pub struct CanvasStateInternal {
    // drag
    is_dragging: bool,
    drag_start_position: Point,
    drag_offset: Point,

    // connection
    is_connecting: bool,
    connection_start: Point,

    // state
    pub camera: Camera,
    pub grid: Grid,
    pub toolbar_state: ToolbarOptions,
    pub graph: StableGraph<Circle, Arrow>,

    // Algorithm state
    pub step_solved: bool,
    algo_state: FluerryState,
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
enum FluerryState {
    NotStarted,
    Initializing,
    CheckingOutgoing,
    ChoosingNext,
    Advancing,
    Backtracking,
    Completed,
    Failed,
}

impl Default for CanvasStateInternal {
    fn default() -> Self {
        CanvasStateInternal::new()
    }
}

impl CanvasStateInternal {
    pub fn new() -> Self {
        return CanvasStateInternal {
            camera: Camera::new(),
            grid: Grid::new(),
            toolbar_state: ToolbarOptions::new(),
            is_dragging: false,
            drag_start_position: Point { x: 0_f32, y: 0_f32 },
            drag_offset: Point { x: 0_f32, y: 0_f32 },
            is_connecting: false,
            connection_start: Point { x: 0_f32, y: 0_f32 },
            graph: StableGraph::new(),

            step_solved: false,
            algo_state: FluerryState::NotStarted,
            stack: Vec::new(),
            circuit: Vec::new(),
            current_node: None,
            graph_clone: StableGraph::new(),
            current_outgoing: Vec::new(),
            current_idx: 0,
            next_candidate: None,
            visited_edges: Vec::new(),
            step_explanation: String::new(),
        };
    }

    // reset when cursor is out of bounds
    pub fn reset_on_oob(&mut self) {
        self.is_connecting = false;
        self.is_dragging = false;
    }

    pub fn draw_arrow(&self, screen: Point) -> Option<Arrow> {
        if self.is_connecting {
            return Some(Arrow {
                start: self.connection_start,
                end: self.camera.screen_to_world(screen),
                line_width: 10.0,
                arrowhead_size: 30.0,
                offset: 0.0,
                color: Color::WHITE,
            });
        }
        None
    }

    pub fn get_cursor_state(&self) -> mouse::Interaction {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if self.is_dragging {
                    mouse::Interaction::Grabbing
                } else {
                    mouse::Interaction::Grab
                }
            }
            ToolbarOptions::Node => mouse::Interaction::Pointer,
            _ => mouse::Interaction::None,
        }
    }

    // left mouse button
    pub fn handle_left_mouse_released(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if self.is_dragging {
                    self.is_dragging = false;

                    self.camera.apply_drag(self.drag_offset);
                    return helpers::CAPTURED;
                }
            }
            ToolbarOptions::Node => {
                self.is_dragging = false;

                if cursor.is_some() {
                    self.create_new_node_on_grid(cursor.unwrap());
                    return helpers::CAPTURED;
                }
            }
            ToolbarOptions::Connection => {
                if cursor.is_some() {
                    if self.is_connecting {
                        self.end_connection(cursor.unwrap());
                        return helpers::CAPTURED;
                    }

                    self.start_connection(cursor.unwrap());
                    return helpers::CAPTURED;
                }
            }
        }
        helpers::IGNORED
    }

    pub fn handle_left_mouse_pressed(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Hand => {
                if !self.is_dragging && cursor.is_some() {
                    self.is_dragging = true;
                    self.drag_start_position = cursor.unwrap();
                    return helpers::CAPTURED;
                }
            }
            ToolbarOptions::Node => {
                self.is_dragging = false;
            }
            _ => {}
        };
        helpers::IGNORED
    }

    // middle mouse button
    pub fn handle_middle_mouse_pressed(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if !self.is_dragging && cursor.is_some() {
            self.is_dragging = true;
            self.drag_start_position = cursor.unwrap();
            helpers::CAPTURED
        } else {
            helpers::IGNORED
        }
    }

    pub fn handle_middle_mouse_released(
        &mut self,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if self.is_dragging {
            self.is_dragging = false;

            self.camera.apply_drag(self.drag_offset);
            helpers::CAPTURED
        } else {
            helpers::IGNORED
        }
    }

    // right mouse button
    pub fn handle_right_mouse_release(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        match self.toolbar_state {
            ToolbarOptions::Node => {
                if cursor.is_some() {
                    self.remove_node_from_grid(cursor.unwrap());

                    return helpers::CAPTURED;
                }
            }
            _ => {}
        };
        helpers::IGNORED
    }

    pub fn handle_mouse_moved(
        &mut self,
        cursor: Option<Point>,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        if self.is_dragging && cursor.is_some() {
            let drag_start: Point = self.drag_start_position;
            self.drag_start_position = cursor.unwrap();
            self.drag_offset = Point {
                x: drag_start.x - cursor.unwrap().x,
                y: drag_start.y - cursor.unwrap().y,
            };

            self.camera.apply_drag(self.drag_offset);
            helpers::CAPTURED
        } else {
            helpers::IGNORED
        }
    }

    fn create_new_node_on_grid(&mut self, screen: Point) {
        let grid_pos = self.grid.to_grid(self.camera.screen_to_world(screen));

        let object = Circle {
            center: grid_pos,
            radius: 30_f32,
            color: Theme::Dark.palette().primary,
        };

        match self.grid.get_object_in_world(grid_pos) {
            Some(_) => {}
            None => {
                let node_index = self.graph.add_node(object);

                self.grid.add_to_grid(grid_pos, node_index);
            }
        }
    }

    fn remove_node_from_grid(&mut self, screen: Point) {
        let grid_pos = self.grid.to_grid(self.camera.screen_to_world(screen));

        let obj = self.grid.get_object_in_world(grid_pos);

        match obj.cloned() {
            // if object is present on grid
            Some(idx) => {
                match self.graph.node_weight(idx) {
                    // if object also present in graph
                    Some(_) => {
                        // remove node itself
                        self.grid.remove_from_grid(grid_pos);

                        self.graph.remove_node(idx.clone());

                        // remove edges
                    }
                    None => {
                        // odd
                        println!("Error: object is not present in grid");

                        self.grid
                            .remove_from_grid(self.camera.screen_to_world(screen));
                    }
                }
            }
            None => {}
        }
    }

    fn start_connection(&mut self, screen: Point) {
        let grid_point = self.grid.to_grid(self.camera.screen_to_world(screen));

        let start_node = self.grid.get_object_in_world(grid_point);

        match start_node {
            Some(_) => {
                self.is_connecting = true;
                self.connection_start = grid_point;
            }
            None => {}
        }
    }

    fn end_connection(&mut self, screen: Point) {
        let start_node = self.grid.get_object_in_world(self.connection_start);
        let end_node = self
            .grid
            .get_object_in_world(self.camera.screen_to_world(screen));

        if start_node.is_some() && end_node.is_some() {
            self.graph.add_edge(
                start_node.unwrap().clone(),
                end_node.unwrap().clone(),
                Arrow {
                    start: self.grid.to_grid(self.connection_start),
                    end: self.grid.to_grid(self.camera.screen_to_world(screen)),
                    line_width: 5.0,
                    arrowhead_size: 10.0,
                    offset: 30.0,
                    color: Color::WHITE,
                },
            );
        }

        self.is_connecting = false;
    }
    pub fn step_algorithm(&mut self) {
        let cur_state = self.algo_state;

        match self.algo_state {
            FluerryState::NotStarted => self.initialize_algorithm(),
            FluerryState::Initializing => self.find_start_node(),
            FluerryState::CheckingOutgoing => self.check_outgoing_edges(),
            FluerryState::ChoosingNext => self.choose_next_edge(),
            FluerryState::Advancing => self.advance_to_next(),
            FluerryState::Backtracking => self.backtrack(),
            FluerryState::Completed | FluerryState::Failed => {
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

    fn initialize_algorithm(&mut self) {
        self.reset_algorithm();
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
            self.algo_state = FluerryState::CheckingOutgoing;
            self.step_explanation = format!("Starting from node {:?}", start_node);
        } else {
            self.algo_state = FluerryState::Completed;
            self.step_explanation = "No suitable start node found - graph may be empty".to_string();
        }
    }

    fn check_outgoing_edges(&mut self) {
        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_state = FluerryState::Failed;
                self.step_explanation = "No current node - algorithm failed".to_string();
                return;
            }
        };

        self.graph
            .node_weight_mut(current)
            .unwrap()
            .highlight_current();

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
                self.algo_state = FluerryState::Completed;
                self.step_explanation = "Algorithm completed - Eulerian circuit found".to_string();
            } else {
                self.algo_state = FluerryState::Failed;
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
            self.algo_state = FluerryState::Backtracking;
            self.step_explanation = "No outgoing edges available - backtracking".to_string();
        } else if self.current_outgoing.len() == 1 {
            // Only one choice - take it
            self.next_candidate = Some(self.current_outgoing[0].0);
            self.algo_state = FluerryState::Advancing;
            self.step_explanation = "Only one outgoing edge - taking it".to_string();
        } else {
            // Multiple choices - need to choose non-bridge if possible
            self.current_idx = 0;
            self.algo_state = FluerryState::ChoosingNext;
            self.step_explanation = format!(
                "Multiple outgoing edges ({}) - checking for non-bridge",
                self.current_outgoing.len()
            );
        }
    }

    fn choose_next_edge(&mut self) {
        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_state = FluerryState::Failed;
                return;
            }
        };

        if self.current_idx >= self.current_outgoing.len() {
            // No non-bridge found, use first edge
            self.next_candidate = Some(self.current_outgoing[0].0);
            self.algo_state = FluerryState::Advancing;
            self.step_explanation = "No non-bridge edge found - using first available".to_string();
            return;
        }

        let (candidate_node, edge_id) = self.current_outgoing[self.current_idx];

        // Highlight current candidate
        self.graph
            .node_weight_mut(candidate_node)
            .unwrap()
            .highlight_possibility();

        println!("{}", self.is_bridge(current, candidate_node));
        if self.is_bridge(current, candidate_node) {
            self.graph
                .edge_weight_mut(edge_id)
                .unwrap()
                .highlight_bridge();
            self.step_explanation = format!("Edge to {:?} is a bridge - skipping", candidate_node);
        } else {
            self.graph
                .edge_weight_mut(edge_id)
                .unwrap()
                .highlight_current();
            self.next_candidate = Some(candidate_node);
            self.algo_state = FluerryState::Advancing;
            self.step_explanation = format!("Found non-bridge edge to {:?}", candidate_node);
            return;
        }

        if self.current_idx < self.current_outgoing.len() {
            self.step_explanation = format!(
                "Checking next candidate ({}/{})",
                self.current_idx + 1,
                self.current_outgoing.len()
            );
        }
        self.current_idx += 1;
    }

    fn advance_to_next(&mut self) {
        let current = match self.current_node {
            Some(node) => node,
            None => {
                self.algo_state = FluerryState::Failed;
                return;
            }
        };

        let next = match self.next_candidate {
            Some(node) => node,
            None => {
                self.algo_state = FluerryState::Failed;
                return;
            }
        };

        // Find and remove the edge
        if let Some(edge_id) = self.graph_clone.find_edge(current, next) {
            self.graph_clone.remove_edge(edge_id);
            self.visited_edges.push(edge_id);

            // Update original graph for visualization
            if let Some(edge_weight) = self.graph.edge_weight_mut(edge_id) {
                edge_weight.highlight_solution();
            }
        }

        // Move to next node
        self.stack.push(current);
        if let Some(cur_weight) = self.graph.node_weight_mut(self.current_node.unwrap()) {
            cur_weight.highlight_solution();
        }
        self.current_node = Some(next);
        self.circuit.push(next);

        self.current_outgoing.clear();
        self.next_candidate = None;
        self.algo_state = FluerryState::CheckingOutgoing;

        self.step_explanation = format!("Moved from {:?} to {:?}", current, next);
    }

    fn backtrack(&mut self) {
        if let Some(prev_node) = self.stack.pop() {
            self.current_node = Some(prev_node);
            self.circuit.push(prev_node);
            self.algo_state = FluerryState::CheckingOutgoing;
            self.step_explanation = format!("Backtracked to node {:?}", prev_node);
        } else {
            self.algo_state = FluerryState::Failed;
            self.step_explanation = "Cannot backtrack - stack is empty".to_string();
        }
    }

    fn clear_highlights(&mut self) {
        for node in self.graph.node_weights_mut() {
            node.reset_highlight();
        }
        for edge in self.graph.edge_weights_mut() {
            edge.reset_highlight();
        }
    }

    pub fn reset_algorithm(&mut self) {
        self.graph_clone = self.graph.clone();
        self.algo_state = FluerryState::Initializing;
        self.stack.clear();
        self.circuit.clear();
        self.current_node = None;
        self.current_outgoing.clear();
        self.current_idx = 0;
        self.next_candidate = None;
        self.visited_edges.clear();
        self.step_explanation.clear();
        self.clear_highlights();
    }

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
