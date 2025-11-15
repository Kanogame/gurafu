use std::collections::HashMap;

use iced::{
    Color, Length, Point, Rectangle, Renderer, Theme, Vector,
    mouse::{self, ScrollDelta},
    widget::{
        self,
        canvas::{self},
    },
};
use petgraph::{
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};
use serde::{Deserialize, Serialize};

use crate::gurafu_app::{
    Node,
    canvas::{
        canvas_frame::CanvasFrame,
        drawable::{DrawablePath, DrawableText, arrow::Arrow, circle::Circle},
        hierholzer::HierholzerState,
        grid::Grid,
    },
    toolbar::ToolbarOptions,
};

mod camera;
mod canvas_frame;
mod drawable;
mod hierholzer;
mod grid;
mod helpers;
// just zoom, pan and interaction state
mod interactions;

#[derive(Clone)]
pub struct CanvasState {
    pub toolbar_state: ToolbarOptions,
    //canvas_cache: Cache,
    pub graph: StableGraph<Circle, Arrow>,
    pub grid: Grid,

    // algo
    algo: HierholzerState,

    // connection
    is_connecting: bool,
    connection_start: Point,
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

pub enum AlgorithmMessage {
    AlgorithmSuccess(Vec<usize>),
    AlgorithmFail,
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
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = CanvasFrame::new(renderer, bounds.size(), state.camera.clone());

        //let cnt = state.graph.node_count() + state.graph.edge_count();
        //let points = state.grid.get_grid_points();

        // fill grid points (alignment helpers)
        for point in state.get_grid_points() {
            frame.fill(point, Color::from_rgb8(120, 120, 120));
        }

        let mut drawable_path_list: Vec<&dyn DrawablePath>;
        let drawable_text_list: Vec<&dyn DrawableText>;

        drawable_path_list = self
            .graph
            .node_references()
            .map(|(_, c)| c as &dyn DrawablePath)
            .collect();

        drawable_text_list = self
            .graph
            .node_references()
            .map(|(_, c)| c as &dyn DrawableText)
            .collect();

        drawable_path_list.append(
            &mut self
                .graph
                .edge_references()
                .map(|el| el.weight() as &dyn DrawablePath)
                .collect(),
        );
        frame.fill_frame(drawable_path_list);
        frame.fill_text(drawable_text_list);

        match cursor.position_in(bounds) {
            Some(c) => match self.draw_arrow(state.convert_screen_to_world(c)) {
                Some(arrow) => {
                    frame.fill(
                        &arrow.into_path(&state.camera),
                        Color::from_rgb8(80, 80, 80),
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
            graph: StableGraph::new(),
            grid: Grid::new(),

            algo: HierholzerState::new(),

            is_connecting: false,
            connection_start: Point { x: 0_f32, y: 0_f32 },
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
                color: Color::from_rgb8(80, 80, 80),
            });
        }
        None
    }

    pub fn create_new_node_on_grid(&mut self, world: Point) {
        let grid_pos = self.grid.to_grid(world);

        let object = Circle {
            id: 0,
            center: grid_pos,
            radius: 30_f32,
            color: Theme::Light.palette().primary,
        };

        match self.grid.get_object_in_world(grid_pos) {
            Some(_) => {}
            None => {
                let node_index = self.graph.add_node(object);

                self.graph.node_weight_mut(node_index).unwrap().id = node_index.index();

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
                    color: Color::from_rgb8(80, 80, 80),
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

    pub fn view(state: &CanvasState) -> iced::Element<'_, CanvasMessage> {
        widget::canvas(state)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn step_algorithm(&mut self) -> Option<AlgorithmMessage> {
        return self.algo.step_algorithm(&mut self.graph);
    }

    pub fn reset_algorithm(&mut self) {
        self.algo.reset_algorithm(&mut self.graph);
    }
}

#[derive(Deserialize, Serialize)]
pub struct CanvasSerializable {
    pub graph: StableGraph<Node, u32>,
}

impl CanvasSerializable {
    pub fn new() -> Self {
        return CanvasSerializable {
            graph: StableGraph::new(),
        };
    }
}

impl Into<StableGraph<Circle, Arrow>> for CanvasSerializable {
    fn into(self) -> StableGraph<Circle, Arrow> {
        let mut res = StableGraph::new();
        let mut node_map: HashMap<NodeIndex, (NodeIndex, &Node)> = HashMap::new();

        self.graph.node_references().for_each(|(idx, node)| {
            let new_idx = res.add_node(node.into_cricle());
            res.node_weight_mut(new_idx).unwrap().id = new_idx.index();

            node_map.insert(idx, (idx, node));
        });

        self.graph.edge_references().for_each(|eref| {
            let src = *node_map.get(&eref.source()).unwrap();
            let trg = *node_map.get(&eref.target()).unwrap();

            res.add_edge(src.0, trg.0, Arrow::from_nodes(src.1, trg.1));
        });

        res
    }
}

impl Into<CanvasSerializable> for CanvasState {
    fn into(self) -> CanvasSerializable {
        let mut res: StableGraph<Node, u32> = StableGraph::new();
        let mut node_map: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        self.graph.node_references().for_each(|(idx, cl)| {
            node_map.insert(idx, res.add_node(cl.center.into()));
        });

        self.graph.edge_references().for_each(|eref| {
            res.add_edge(
                *node_map.get(&eref.source()).unwrap(),
                *node_map.get(&eref.target()).unwrap(),
                0,
            );
        });

        CanvasSerializable { graph: res }
    }
}
