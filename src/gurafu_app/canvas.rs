use iced::{
    Color, Event, Point, Rectangle, Renderer, Theme, Vector,
    mouse::{self, ScrollDelta},
    widget::{
        self,
        canvas::{self, Cache},
    },
    window,
};
use petgraph::{
    Direction,
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};

use crate::gurafu_app::{
    canvas::{
        canvas_frame::CanvasFrame,
        drawable::{Drawable, arrow::Arrow, circle::Circle},
        grid::Grid,
    },
    toolbar::ToolbarOptions,
};

mod camera;
mod canvas_frame;
mod drawable;
mod grid;
mod helpers;
// just zoom, pan and interaction state
mod interactions;

#[derive(Clone)]
pub struct CanvasState {
    pub toolbar_state: ToolbarOptions,
    pub solving_required: bool,
    //canvas_cache: Cache,
    pub graph: StableGraph<Circle, Arrow>,
    pub grid: Grid,

    // connection
    is_connecting: bool,
    connection_start: Point,

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

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    // Point - point in world

    // create new node on grid at point
    CreateNodeOnGrid(Point),

    // remove node from closest grid point to point
    RemoveNodeFromGrid(Point),

    // connect elements
    HandleConnection(Point),
}

impl canvas::Program<CanvasMessage> for CanvasState {
    type State = interactions::CanvasStateInternal;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<CanvasMessage>) {
        state.toolbar_state = self.toolbar_state.clone();
        let pos = cursor.position_in(bounds);

        if pos.is_none() {
            state.reset_on_oob();
        }

        state.update_grid_points(&self.grid, bounds.size());

        match event {
            // although implementing state mutation in a canvas defies Elm arch,
            // propagating it higher would be a giant overhead, so we act
            // like a widget here and handle our state internally, exposing only
            // minimal info needed (currently none)
            //
            // Btw, this is the "inteded way"
            canvas::Event::Mouse(m_ev) => match m_ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.handle_left_mouse_pressed(pos)
                }
                mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                    state.handle_middle_mouse_pressed(pos)
                }
                mouse::Event::ButtonReleased(button) => match button {
                    mouse::Button::Left => state.handle_left_mouse_released(pos),
                    mouse::Button::Right => state.handle_right_mouse_release(pos),
                    mouse::Button::Middle => state.handle_middle_mouse_released(),
                    _ => helpers::IGNORED,
                },
                mouse::Event::CursorMoved { position: _ } => state.handle_mouse_moved(pos),
                mouse::Event::WheelScrolled { delta } => {
                    let delta_vec: Vector = match delta {
                        ScrollDelta::Lines { x, y } => Vector { x: x, y: y },
                        ScrollDelta::Pixels { x, y } => Vector { x: x, y: y },
                    };

                    state.camera.apply_scroll(delta_vec.y / 4_f32);
                    state.update_grid_points(&self.grid, bounds.size());

                    return helpers::CAPTURED;
                }
                _ => helpers::IGNORED,
            },
            _ => helpers::IGNORED,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = CanvasFrame::new(renderer, bounds.size(), state.camera.clone());

        //let cnt = state.graph.node_count() + state.graph.edge_count();
        //let points = state.grid.get_grid_points();

        // fill grid points (alignment helpers)
        for point in state.get_grid_points() {
            frame.fill(point, theme.palette().success);
        }

        let mut drawable_list: Vec<&dyn Drawable>;

        drawable_list = self
            .graph
            .node_references()
            .map(|(_, c)| c as &dyn Drawable)
            .collect();
        drawable_list.append(
            &mut self
                .graph
                .edge_references()
                .map(|el| el.weight() as &dyn Drawable)
                .collect(),
        );
        frame.fill_frame(drawable_list);

        match cursor.position_in(bounds) {
            Some(c) => match self.draw_arrow(state.convert_screen_to_world(c)) {
                Some(arrow) => {
                    frame.fill(
                        &arrow.into_path(&state.camera),
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                    );
                }
                None => {}
            },
            None => {
                // cursor is outside the canvas, do nothing
            }
        }

        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        state.get_cursor_state()
    }
}

impl CanvasState {
    pub fn new() -> Self {
        CanvasState {
            toolbar_state: ToolbarOptions::new(),
            solving_required: false,
            graph: StableGraph::new(),
            grid: Grid::new(),

            is_connecting: false,
            connection_start: Point { x: 0_f32, y: 0_f32 },

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
        }
    }

    pub fn draw_arrow(&self, end: Point) -> Option<Arrow> {
        if self.is_connecting {
            return Some(Arrow {
                start: self.connection_start,
                end: end,
                line_width: 10.0,
                arrowhead_size: 30.0,
                offset: 0.0,
                color: Color::WHITE,
            });
        }
        None
    }

    pub fn create_new_node_on_grid(&mut self, world: Point) {
        let grid_pos = self.grid.to_grid(world);

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

    pub fn remove_node_from_grid(&mut self, world: Point) {
        let grid_pos = self.grid.to_grid(world);

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

                        self.grid.remove_from_grid(world);
                    }
                }
            }
            None => {}
        }
    }

    pub fn start_connection(&mut self, world: Point) {
        let grid_point = self.grid.to_grid(world);

        let start_node = self.grid.get_object_in_world(grid_point);

        match start_node {
            Some(_) => {
                self.is_connecting = true;
                self.connection_start = grid_point;
            }
            None => {}
        }
    }

    pub fn end_connection(&mut self, world: Point) {
        let start_node = self.grid.get_object_in_world(self.connection_start);
        let end_node = self.grid.get_object_in_world(world);

        if start_node.is_some() && end_node.is_some() {
            self.graph.add_edge(
                start_node.unwrap().clone(),
                end_node.unwrap().clone(),
                Arrow {
                    start: self.grid.to_grid(self.connection_start),
                    end: self.grid.to_grid(world),
                    line_width: 5.0,
                    arrowhead_size: 10.0,
                    offset: 30.0,
                    color: Color::WHITE,
                },
            );
        }

        self.is_connecting = false;
    }

    pub fn handle_connection(&mut self, world: Point) {
        if self.is_connecting {
            self.end_connection(world);
        } else {
            self.start_connection(world);
        }
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

        self.current_idx += 1;

        if self.current_idx < self.current_outgoing.len() {
            self.step_explanation = format!(
                "Checking next candidate ({}/{})",
                self.current_idx + 1,
                self.current_outgoing.len()
            );
        }
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

    pub fn view(state: &CanvasState) -> iced::Element<'_, CanvasMessage> {
        widget::canvas(state).into()
    }
}
