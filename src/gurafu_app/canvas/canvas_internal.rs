use std::collections::{HashSet, VecDeque};

use iced::{Point, mouse, widget::canvas};
use petgraph::{
    Direction::{Incoming, Outgoing},
    graph::NodeIndex,
    prelude::StableGraph,
    visit::EdgeCount,
};

use crate::gurafu_app::{
    canvas::{
        CanvasMessage,
        camera::Camera,
        canvas_internal::grid::Grid,
        drawable::{Arrow, Circle},
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
                },
            );
        }

        self.is_connecting = false;
    }

    fn is_strongly_connected_nonzero(&self) -> bool {
        let g = &self.graph;

        // Quick check: reachable set from a start vertex with nonzero deg should include all vertices with nonzero degree
        let start = g.node_indices().find(|&n| {
            g.neighbors_directed(n, Outgoing).next().is_some()
                || g.neighbors_directed(n, Incoming).next().is_some()
        });
        if start.is_none() {
            return true;
        } // empty graph trivially
        let start = start.unwrap();

        // BFS on directed graph following outgoing edges (to ensure single strongly-connected component is more costly;
        // for Euler circuit we need strongly connected ignoring direction — here we'll check reachability on underlying undirected edges)
        let mut visited = HashSet::new();
        let mut q = VecDeque::new();
        visited.insert(start);
        q.push_back(start);
        while let Some(v) = q.pop_front() {
            for nbr in g.neighbors_directed(v, Outgoing) {
                if !visited.contains(&nbr) {
                    visited.insert(nbr);
                    q.push_back(nbr);
                }
            }
            for nbr in g.neighbors_directed(v, Incoming) {
                if !visited.contains(&nbr) {
                    visited.insert(nbr);
                    q.push_back(nbr);
                }
            }
        }

        // all vertices with nonzero degree must be in visited
        for v in g.node_indices() {
            if g.neighbors_directed(v, Outgoing).next().is_some()
                || g.neighbors_directed(v, Incoming).next().is_some()
            {
                if !visited.contains(&v) {
                    return false;
                }
            }
        }
        true
    }

    fn would_be_bridge(&self, u: NodeIndex, v: NodeIndex) -> bool {
        // Remove edge (u->v) temporarily and check if v is still reachable from u using directed edges.
        // If not reachable, edge is a bridge (necessary for connectivity of remaining traversal).
        let mut temp = self.graph.clone();
        temp.remove_edge(temp.find_edge(u, v).unwrap());

        // BFS/DFS from u following outgoing edges to see if we can reach v (consider only nodes with remaining degree)
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        visited.insert(u);
        stack.push(u);
        while let Some(x) = stack.pop() {
            for nbr in temp.neighbors_directed(x, Outgoing) {
                if !visited.contains(&nbr) {
                    visited.insert(nbr);
                    stack.push(nbr);
                }
            }
        }
        visited.contains(&v)
    }

    pub fn solve_flurry(&self, start: NodeIndex) -> Option<Vec<NodeIndex>> {
        let mut g = self.graph.clone();

        // Basic prechecks
        // in-degree == out-degree for all vertices
        for n in g.node_indices() {
            let out = g.neighbors_directed(n, Outgoing).count();
            let inp = g.neighbors_directed(n, Incoming).count();
            if out != inp {
                return None;
            }
        }
        if !self.is_strongly_connected_nonzero() {
            return None;
        }

        let mut circuit = Vec::new();
        let mut current = start;
        circuit.push(current);

        // Maintain adjacency counts (multiedges not supported by GraphMap; if needed, use Graph and edge indices)
        while g.neighbors_directed(current, Outgoing).next().is_some() {
            // gather outgoing edges from current
            let outs: Vec<NodeIndex> = g.neighbors_directed(current, Outgoing).collect();

            // choose an edge that is not a bridge if possible
            let mut chosen = None;
            for &v in &outs {
                // if it's the only outgoing edge, must choose it
                if outs.len() == 1 {
                    chosen = Some(v);
                    break;
                }
                // choose v if removing (current->v) does not break reachability
                if self.would_be_bridge(current, v) {
                    // removing (current->v) still leaves v reachable -> NOT a bridge
                    // note: would_be_bridge returns true if v remains reachable, so invert logic
                    chosen = Some(v);
                    break;
                }
            }
            // if not found (all edges are bridges by our check), pick first
            if chosen.is_none() {
                chosen = outs.into_iter().next();
            }
            let v = chosen.unwrap();
            // remove edge and move
            g.remove_edge(g.find_edge(current, v).unwrap());
            current = v;
            circuit.push(current);
        }

        Some(circuit)
    }
}
