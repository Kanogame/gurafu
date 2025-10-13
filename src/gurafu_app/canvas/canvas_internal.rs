
use iced::{mouse, widget::{canvas, }, Color, Point, Theme};
use petgraph::{
    algo, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef, Direction::{self, Incoming, Outgoing}, Graph
};

use crate::gurafu_app::{
    canvas::{
        camera::Camera, canvas_internal::grid::Grid, drawable::{arrow::Arrow, circle::Circle, Drawable}, helpers, CanvasMessage
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

    // algo
    algo_state: FluerryState,
    stack: Vec<NodeIndex>,
    circuit: Vec<NodeIndex>,
    current_node: Option<NodeIndex>,
    graph_clone: StableGraph<Circle, Arrow>,

    current_outgoing: Vec<NodeIndex>,
    current_idx: usize,
    next: NodeIndex,
}

#[derive(Debug)]
enum FluerryState {
    NotStarted,
    InProcess,
    ChoosingNext,
    Advancing,
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

            algo_state: FluerryState::NotStarted,
            stack: Vec::new(),
            circuit: Vec::new(),
            current_node: None,
            graph_clone: StableGraph::new(),
            current_outgoing: Vec::new(),
            current_idx: 0,
            next: NodeIndex::new(0),
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
                color: Color::WHITE
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

    fn create_new_node_on_grid(&mut self, screen: Point,) {
        let grid_pos = self.grid.to_grid(self.camera.screen_to_world(screen));

        let object = Circle {
            center: grid_pos,
            radius: 30_f32,
            color: Theme::Dark.palette().primary
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
                    color: Color::WHITE
                },
            );
        }

        self.is_connecting = false;
    }

    pub fn step_algorithm(&mut self) {
        println!("{:?} {:?} {:?} {:?} {:?}", self.algo_state, self.stack, self.circuit, self.current_node, self.graph_clone);

        match self.algo_state {
            FluerryState::NotStarted => {
                // 1. clone graph
                self.graph_clone = self.graph.clone();

                // 2. find start node (any node with outgoing edges)
                self.current_node = self.graph_clone.node_indices()
                .find(|&node| self.graph_clone.edges_directed(node, Direction::Outgoing).count() > 0);

                // 3. clear stack
                self.stack.clear();

                // 4. change state
                self.algo_state = FluerryState::InProcess;
            }
            FluerryState::InProcess => {
                if !self.stack.is_empty() || 
                    self.current_node.is_some_and(|cur| 
                        self.graph_clone.edges_directed(cur, Direction::Outgoing).count() > 0) {

                    // if we found ourself in case when there are no outgoing paths from here, then we backtrack
                    if self.graph_clone.edges_directed(self.current_node.unwrap(), Direction::Outgoing).count() == 0 {
                        // Backtrack
                        self.current_node = self.stack.pop();
                        self.circuit.push(self.current_node.unwrap());
                        println!("backtrack");
                    } else {
                        // Move forward
                        // Pushing current to stack
                        self.stack.push(self.current_node.unwrap());
                        self.graph.node_weight_mut(self.current_node.unwrap()).unwrap().highlight_solution();
                        
                        // Get all indexes of outgoing edges of current
                        self.current_outgoing = self.graph_clone
                            .edges_directed(self.current_node.unwrap(), Direction::Outgoing)
                            .map(|edge| edge.target())
                            .collect();

                        // If we have only one option, we choose it as next node
                        if self.current_outgoing.len() == 1 {
                            // Advancing towards self.current_outgoing[0]
                            self.next = self.current_outgoing[0];
                            self.algo_state = FluerryState::Advancing;
                        } else {
                            // For multiple options, pick one that's not a bridge if possible
                             self.current_idx = 0;
                            self.algo_state = FluerryState::ChoosingNext;
                        }
                    }
                }
            }
            FluerryState::ChoosingNext => {           
                // For multiple options, pick one that's not a bridge if possible
                if self.current_idx >= self.current_outgoing.len() {
                    self.algo_state = FluerryState::Failed;
                    return;
                }

                self.graph.node_weight_mut(self.current_outgoing[self.current_idx]).unwrap().highlight_possibility();

                if !self.is_bridge(self.current_node.unwrap(), self.current_outgoing[self.current_idx]) {
                    self.next = self.current_outgoing[self.current_idx];
                    self.algo_state = FluerryState::Advancing;
                }

                self.current_idx += 1;
            }
            FluerryState::Advancing => {
                self.graph.node_weight_mut(self.next).unwrap().highlight_solution();

                // Remove the edge and move
                if let Some(edge_id) = self.graph_clone.find_edge(self.current_node.unwrap(), next) {
                    self.graph_clone.remove_edge(edge_id);
                    self.graph.edge_weight_mut(edge_id).unwrap().highlight_solution();
                }
                self.current_node = Some(self.next);
            }

            FluerryState::Failed => {}
        }
    }

    pub fn solve_flurry(&self) -> Option<Vec<NodeIndex>> {
        // cloned graph
        let mut graph = self.graph.clone();

        // Find start node (any node with outgoing edges)
        let mut current = graph.node_indices()
            .find(|&node| graph.edges_directed(node, Direction::Outgoing).count() > 0)
            .unwrap_or_else(|| graph.node_indices().next().unwrap_or(NodeIndex::new(0)));

        let mut circuit = Vec::new();
        circuit.push(current);

        // Use a stack to avoid recursion limits in large graphs
        let mut stack = Vec::new();
        
        while !stack.is_empty() || graph.edges_directed(current, Direction::Outgoing).count() > 0 {
            if graph.edges_directed(current, Direction::Outgoing).count() == 0 {
                // Backtrack
                current = stack.pop().unwrap();
                circuit.push(current);
            } else {
                // Move forward
                // Pushing current to stack
                stack.push(current);
                
                // Get all indexes of outgoing edges of current
                let edges: Vec<NodeIndex> = graph
                    .edges_directed(current, Direction::Outgoing)
                    .map(|edge| edge.target())
                    .collect();

                // Choose next node
                let next;
                
                // If we have only one option, we choose it as next node
                if edges.len() == 1 {
                    next = edges[0]
                } else {
                    // For multiple options, pick one that's not a bridge if possible
                    next = edges.iter()
                        .find(|&&neighbor| 
                            // check for bridge
                            !self.is_bridge(current, neighbor))
                        .copied()
                        .unwrap_or(edges[0])
                };

                // Remove the edge and move
                if let Some(edge_id) = graph.find_edge(current, next) {
                    graph.remove_edge(edge_id);
                }
                current = next;
            }
        }

        Some(circuit)
    }


    fn is_bridge(&self, u: NodeIndex, v: NodeIndex) -> bool {
        // Simple bridge detection: if removing u->v disconnects the graph
        let mut temp_graph = self.graph.clone();
        
        if let Some(edge_id) = temp_graph.find_edge(u, v) {
            temp_graph.remove_edge(edge_id);
            
            // Check if v is still reachable from u in the undirected sense
            let reachable = algo::dijkstra(&temp_graph, u, Some(v), |_| 1);
            !reachable.contains_key(&v)
        } else {
            false
        }
    }


    pub fn highlight_solution(&mut self, solution: Vec<NodeIndex>) {
        println!("{:?}", self.graph);
        
        // highlight nodes
        for idx in solution.clone() {
            self.graph.node_weight_mut(idx).unwrap().highlight_solution();
        }
        
        // highlight edges
        for els in solution.iter().rev().collect::<Vec<&NodeIndex>>().windows(2) {
            println!("{:?}", els);

            let eidx = self.graph.find_edge(*els[0], *els[1]).unwrap();
            self.graph.edge_weight_mut(eidx).unwrap().highlight_solution();
        }
    }
}
