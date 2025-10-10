use std::collections::{HashSet, VecDeque};

use iced::{mouse, widget::canvas, Color, Point, Theme};
use petgraph::{
    algo, graph::NodeIndex, prelude::StableGraph, visit::EdgeRef, Direction::{self, Incoming, Outgoing}
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

     pub fn solve_flurry(&self) -> Option<Vec<NodeIndex>> {
        let mut graph = self.graph.clone();
        
        // Check if graph has an Eulerian circuit using standard conditions
        if !self.is_eulerian() {
            return None;
        }

        // Find start node (any node with outgoing edges)
        let start = graph.node_indices()
            .find(|&node| graph.edges_directed(node, Direction::Outgoing).count() > 0)
            .unwrap_or_else(|| graph.node_indices().next().unwrap_or(NodeIndex::new(0)));

        let mut circuit = Vec::new();
        let mut current = start;
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
                stack.push(current);
                
                // Get all outgoing edges
                let edges: Vec<NodeIndex> = graph
                    .edges_directed(current, Direction::Outgoing)
                    .map(|edge| edge.target())
                    .collect();

                // Choose next node - try to avoid bridges when possible
                let next = if edges.len() == 1 {
                    edges[0]
                } else {
                    // For multiple choices, pick one that's not a bridge if possible
                    edges.iter()
                        .find(|&&neighbor| !self.is_bridge_simple(current, neighbor))
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

    fn is_eulerian(&self) -> bool {
        let graph = &self.graph;
        
        // Check degree condition: in_degree == out_degree for all vertices
        for node in graph.node_indices() {
            let in_degree = graph.edges_directed(node, Direction::Incoming).count();
            let out_degree = graph.edges_directed(node, Direction::Outgoing).count();
            
            if in_degree != out_degree {
                return false;
            }
        }

        // Check connectivity using petgraph's built-in function
        self.is_weakly_connected()
    }

fn is_weakly_connected(&self) -> bool {
    let graph = &self.graph;
    
    if graph.node_count() == 0 {
        return true;
    }

    let start = graph.node_indices().next().unwrap();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    while let Some(node) = queue.pop_front() {
        for neighbor in graph.neighbors_undirected(node) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    // For Eulerian circuits, isolated vertices don't matter
    // Only check that all vertices with edges are connected
    visited.len() + graph.node_indices()
        .filter(|&node| {
            graph.edges_directed(node, Direction::Outgoing).count() == 0 &&
            graph.edges_directed(node, Direction::Incoming).count() == 0
        })
        .count() == graph.node_count()
}

    fn is_bridge_simple(&self, u: NodeIndex, v: NodeIndex) -> bool {
        // Simple bridge detection: if removing u->v disconnects the graph
        let mut temp_graph = self.graph.clone();
        
        if let Some(edge_id) = temp_graph.find_edge(u, v) {
            temp_graph.remove_edge(edge_id);
            
            // Check if v is still reachable from u in the undirected sense
            let reachable = algo::dijkstra(&temp_graph, u, None, |_| 1);
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
