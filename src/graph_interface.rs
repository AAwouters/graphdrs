use std::time::{Duration, Instant};

use macroquad::{prelude::*, rand};

use crate::{
    graph::Graph,
    graph_drawer::{EdgeDrawConfig, VertexDrawConfig},
    grid::{CircleGrid, SquareGrid},
    ui_manager::main_screen_width,
};

pub struct VertexProperties {
    pub position: Vec2,
    pub radius: f32,
    pub draw_state: DrawState,
}

impl VertexProperties {
    pub fn cycle_drawstate(&mut self) {
        self.draw_state = match self.draw_state {
            DrawState::Default => DrawState::Highlighted,
            DrawState::Highlighted => DrawState::Unhighlighted,
            DrawState::Unhighlighted => DrawState::Hidden,
            DrawState::Hidden => DrawState::Default,
        }
    }
}

impl Default for VertexProperties {
    fn default() -> Self {
        let vertex_config = VertexDrawConfig::default();

        Self {
            position: Vec2::ZERO,
            radius: vertex_config.main_size + vertex_config.border_size,
            draw_state: DrawState::Default,
        }
    }
}

pub struct EdgeProperties {
    pub vertices: (usize, usize),
    pub width: f32,
    pub draw_state: DrawState,
}

impl EdgeProperties {
    pub fn cycle_drawstate(&mut self) {
        self.draw_state = match self.draw_state {
            DrawState::Default => DrawState::Highlighted,
            DrawState::Highlighted => DrawState::Unhighlighted,
            DrawState::Unhighlighted => DrawState::Hidden,
            DrawState::Hidden => DrawState::Default,
        }
    }
}

impl Default for EdgeProperties {
    fn default() -> Self {
        let config = EdgeDrawConfig::default();

        Self {
            vertices: (0, 0),
            width: config.width,
            draw_state: DrawState::Default,
        }
    }
}

#[derive(PartialEq)]
pub enum DrawState {
    Default,
    Highlighted,
    Unhighlighted,
    Hidden,
}

pub struct GraphInterface {
    pub vertex_properties: Vec<VertexProperties>,
    pub edge_properties: Vec<EdgeProperties>,
    pub dragged_vertex: Option<usize>,
    pub hovered_vertex: Option<usize>,
    pub hovered_edge: Option<usize>,
    drag_state: Option<DragState>,
    click_handler: ClickHandler,
    highlight_graph_history: Vec<Graph>,
    pub current_highlight_graph: Option<usize>,
}

impl GraphInterface {
    pub fn new(graph: &Graph) -> Self {
        let center = Vec2::new(main_screen_width() / 2.0, screen_height() / 2.0);
        let tau_part = std::f32::consts::TAU / graph.vertices as f32;

        let mut vertex_properties = Vec::with_capacity(graph.vertices);

        let offset_magnitude = main_screen_width().min(screen_height()) / 2.0 - 50.0;

        for i in 0..graph.vertices {
            let i = i as f32;
            let x_offset = (i * tau_part).sin() * offset_magnitude;
            let y_offset = -(i * tau_part).cos() * offset_magnitude;
            let offset = Vec2::new(x_offset, y_offset);

            let properties = VertexProperties {
                position: center + offset,
                ..Default::default()
            };

            vertex_properties.push(properties);
        }

        let mut edge_properties = Vec::with_capacity(graph.edges.len());

        for edge in &graph.edges {
            let properties = EdgeProperties {
                vertices: *edge,
                ..Default::default()
            };

            edge_properties.push(properties);
        }

        GraphInterface {
            vertex_properties,
            edge_properties,
            dragged_vertex: None,
            hovered_vertex: None,
            drag_state: None,
            hovered_edge: None,
            click_handler: ClickHandler::new(),
            highlight_graph_history: Vec::new(),
            current_highlight_graph: None,
        }
    }

    pub fn update_edges(&mut self, graph: &Graph) {
        let mut edge_properties = Vec::with_capacity(graph.edges.len());

        for edge in &graph.edges {
            let properties = EdgeProperties {
                vertices: *edge,
                ..Default::default()
            };

            edge_properties.push(properties);
        }

        self.edge_properties = edge_properties;
    }

    pub fn get_position(&self, vertex: usize) -> Vec2 {
        self.vertex_properties
            .get(vertex)
            .map(|v| v.to_owned().position)
            .unwrap_or(Vec2::ZERO)
    }

    pub fn set_position(&mut self, vertex: usize, position: Vec2) {
        if vertex < self.vertex_properties.len() {
            self.vertex_properties[vertex].position = position;
        } else {
            for _ in self.vertex_properties.len()..vertex {
                self.vertex_properties.push(VertexProperties::default());
            }
            self.vertex_properties.push(VertexProperties {
                position,
                ..Default::default()
            });
        }
    }

    pub fn get_vertex_at_position(&self, position: Vec2) -> Option<usize> {
        for (index, vertex_properties) in self.vertex_properties.iter().enumerate() {
            let distance = position.distance(vertex_properties.position);

            if distance < vertex_properties.radius {
                return Some(index);
            }
        }

        None
    }

    pub fn get_edge_at_position(&self, position: Vec2) -> Option<usize> {
        for (i, edge_properties) in self.edge_properties.iter().enumerate() {
            let vertices = edge_properties.vertices;
            let start = self.get_position(vertices.0);
            let end = self.get_position(vertices.1);
            let width = edge_properties.width;

            let (min_x, max_x) = (start.x.min(end.x) - width, start.x.max(end.x) + width);
            let (min_y, max_y) = (start.y.min(end.y) - width, start.y.max(end.y) + width);

            // mouse position is within bounding box of line
            if (min_x <= position.x && position.x <= max_x)
                && (min_y <= position.y && position.y <= max_y)
            {
                let distance = distance_to_line(start, end, position);

                if distance < width {
                    return Some(i);
                }
            }
        }

        None
    }

    pub fn handle_mouse_input(&mut self) {
        let mouse_position: Vec2 = mouse_position().into();
        self.click_handler.register_mouse_button_status();

        // Dragging vertex
        if let Some(previous_drag_state) = self.drag_state {
            let dragged_vertex = previous_drag_state.vertex;

            // Still dragging
            if self.click_handler.mouse_drag() {
                let delta = mouse_position - previous_drag_state.mouse_position;
                let old_pos = self.get_position(dragged_vertex);
                let new_pos = old_pos + delta;
                self.set_position(dragged_vertex, new_pos);

                self.drag_state = Some(DragState {
                    vertex: dragged_vertex,
                    mouse_position,
                });

            // Stop dragging
            } else {
                self.drag_state = None;
                self.dragged_vertex = None;
            }
        }
        // Not dragging vertex
        else {
            let hovered_vertex = self.get_vertex_at_position(mouse_position);
            let hovered_edge = self.get_edge_at_position(mouse_position);

            // Highlight hovered vertex
            if !self.click_handler.mouse_drag() {
                self.hovered_vertex = hovered_vertex;
            // Possibly start dragging vertex
            } else {
                self.hovered_vertex = None;

                if let Some(dragged_vertex) = hovered_vertex {
                    self.dragged_vertex = Some(dragged_vertex);
                    self.drag_state = Some(DragState {
                        vertex: dragged_vertex,
                        mouse_position,
                    })
                }
            }

            if hovered_vertex.is_none() {
                self.hovered_edge = hovered_edge;
            } else {
                self.hovered_edge = None;
            }

            if self.click_handler.mouse_click() {
                if let Some(hovered_vertex) = self.hovered_vertex {
                    let vertex_properties = self.vertex_properties.get_mut(hovered_vertex).unwrap();

                    vertex_properties.cycle_drawstate();
                }

                if let Some(hovered_edge) = self.hovered_edge {
                    let edge_properties = self.edge_properties.get_mut(hovered_edge).unwrap();

                    edge_properties.cycle_drawstate();
                }
            }
        }
    }

    pub fn apply_force(&mut self, graph: &Graph) {
        let mut forces: Vec<Vec2> = Vec::with_capacity(graph.vertices);

        // calculate forces
        for main_vertex in 0..graph.vertices {
            let mut total_force = Vec2::ZERO;

            let main_position = self.get_position(main_vertex);

            for secondary_vertex in 0..graph.vertices {
                if secondary_vertex == main_vertex {
                    continue;
                }

                let secondary_position = self.get_position(secondary_vertex);
                let distance = main_position.distance(secondary_position);

                let force = if distance == 0.0 {
                    vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0))
                } else {
                    let direction = (secondary_position - main_position).normalize();

                    let edge = (
                        main_vertex.min(secondary_vertex),
                        main_vertex.max(secondary_vertex),
                    );

                    let magnitude = if graph.edges.contains(&edge) {
                        (distance / 70.0).log10()
                    } else {
                        -50.0 * 0.5f32.powf(distance / 20.0)
                    };

                    direction * magnitude
                };

                total_force += force;
            }

            forces.push(total_force);
        }

        // apply forces
        self.apply_forces(&forces);
    }

    pub fn align_to_square_grid(&mut self, grid: &SquareGrid) {
        fn parabole(x: f32, top_x: f32) -> f32 {
            let x = x / (2.0 * top_x);
            4.0 * (x - x * x)
        }

        let mut forces: Vec<Vec2> = Vec::with_capacity(self.vertex_properties.len());

        let delta_avg = grid.delta_avg();

        for vertex in self.vertex_properties.iter() {
            let vertex_pos = vertex.position;
            let closest_grid_point = Vec2::new(
                ((vertex_pos.x - grid.x_offset) / grid.x_delta).round() * grid.x_delta
                    + grid.x_offset,
                ((vertex_pos.y - grid.y_offset) / grid.y_delta).round() * grid.y_delta
                    + grid.y_offset,
            );
            let distance = closest_grid_point.distance(vertex_pos);

            let force = if distance > 0.0 {
                let direction = (closest_grid_point - vertex_pos).normalize();
                let magnitude = 5.0 * parabole(distance, delta_avg);
                direction * magnitude
            } else {
                Vec2::ZERO
            };

            forces.push(force);
        }

        self.apply_forces(&forces);
    }

    pub fn align_to_circular_grid(&mut self, grid: &CircleGrid) {
        fn parabole(x: f32, top_x: f32) -> f32 {
            let x = x / (2.0 * top_x);
            4.0 * (x - x * x)
        }

        let mut forces: Vec<Vec2> = Vec::with_capacity(self.vertex_properties.len());

        for vertex in self.vertex_properties.iter() {
            let vertex_pos = vertex.position;

            let force = if let Some(direction) = (vertex_pos - grid.center).try_normalize() {
                let center_distance = grid.center.distance(vertex_pos);
                let distance_mod = center_distance % grid.r_delta;
                let sign = if distance_mod - 0.5 * grid.r_delta >= 0.0 {
                    1.0
                } else {
                    -1.0
                };

                let magnitude = 5.0 * parabole(distance_mod, 0.5 * grid.r_delta);

                sign * magnitude * direction
            } else {
                Vec2::ZERO
            };

            forces.push(force);
        }

        self.apply_forces(&forces);
    }

    pub fn apply_forces(&mut self, forces: &[Vec2]) {
        if forces.len() != self.vertex_properties.len() {
            debug!("forces and vertex properties arrays not of same length");
            return;
        }

        for (vertex, force) in forces.iter().enumerate() {
            if let Some(dragged) = self.dragged_vertex {
                if dragged == vertex {
                    continue;
                }
            }

            let old_position = self.get_position(vertex);
            let new_position = old_position + *force;
            let clamped_position =
                new_position.clamp(vec2(0.0, 0.0), vec2(main_screen_width(), screen_height()));

            self.set_position(vertex, clamped_position);
        }
    }

    pub fn clear_edge_highlighting(&mut self) {
        for edge in self.edge_properties.iter_mut() {
            edge.draw_state = DrawState::Default;
        }
        self.current_highlight_graph = None;
    }

    pub fn add_edge_highlighting(&mut self, edges: &[(usize, usize)]) {
        for edge_properties in self.edge_properties.iter_mut() {
            if edges.contains(&edge_properties.vertices) {
                edge_properties.draw_state = DrawState::Highlighted;
            }
        }
    }

    pub fn set_edge_highlighting(&mut self, edges: &[(usize, usize)]) {
        self.clear_edge_highlighting();
        self.add_edge_highlighting(edges);
    }

    pub fn clear_highlight_history(&mut self) {
        self.highlight_graph_history.clear();
        self.clear_edge_highlighting();
    }

    pub fn add_graph_to_history(&mut self, graph: Graph) {
        self.highlight_graph_history.push(graph);
    }

    pub fn get_history_size(&self) -> usize {
        self.highlight_graph_history.len()
    }

    pub fn set_edge_highlighting_and_add_to_history(&mut self, graph: Graph) {
        self.set_edge_highlighting(&graph.edges);
        self.add_graph_to_history(graph);
        self.current_highlight_graph = Some(self.highlight_graph_history.len() - 1);
    }

    pub fn set_next_highlighting(&mut self) {
        let highlight_vec = &self.highlight_graph_history;
        let target_index = match self.current_highlight_graph {
            Some(index) => index + 1,
            None => 0,
        };

        if target_index < highlight_vec.len() {
            let graph = &highlight_vec[target_index];
            self.current_highlight_graph = Some(target_index);
            for edge_properties in self.edge_properties.iter_mut() {
                if graph.edges.contains(&edge_properties.vertices) {
                    edge_properties.draw_state = DrawState::Highlighted;
                } else {
                    edge_properties.draw_state = DrawState::Default;
                }
            }
        }
    }

    pub fn set_previous_highlighting(&mut self) {
        let highlight_vec = &self.highlight_graph_history;
        let target_index = match self.current_highlight_graph {
            Some(index) => index - 1,
            None => highlight_vec.len() - 1,
        };

        if target_index < highlight_vec.len() {
            let graph = &highlight_vec[target_index];
            self.current_highlight_graph = Some(target_index);
            for edge_properties in self.edge_properties.iter_mut() {
                if graph.edges.contains(&edge_properties.vertices) {
                    edge_properties.draw_state = DrawState::Highlighted;
                } else {
                    edge_properties.draw_state = DrawState::Default;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct DragState {
    vertex: usize,
    mouse_position: Vec2,
}

struct ClickHandler {
    click_start: Option<(Instant, Vec2)>,
    drag_min_duration: Duration,
    registered_click_this_frame: bool,
}

impl ClickHandler {
    fn new() -> Self {
        Self {
            click_start: None,
            drag_min_duration: Duration::from_millis(125),
            registered_click_this_frame: false,
        }
    }

    fn register_mouse_button_status(&mut self) {
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        self.registered_click_this_frame = false;

        if mouse_down {
            if self.click_start.is_none() {
                self.click_start = Some((Instant::now(), mouse_position().into()))
            }
        } else if let Some(start) = self.click_start {
            let time_since_start = Instant::now() - start.0;

            self.registered_click_this_frame = time_since_start < self.drag_min_duration;

            self.click_start = None;
        }
    }

    fn mouse_drag(&self) -> bool {
        if let Some(start) = self.click_start {
            let time_since_start = Instant::now() - start.0;
            let drag_distance = start.1.distance(mouse_position().into());

            return time_since_start > self.drag_min_duration || drag_distance > 5.0;
        }

        false
    }

    fn mouse_click(&self) -> bool {
        self.registered_click_this_frame
    }
}

fn distance_to_line(line_start: Vec2, line_end: Vec2, point: Vec2) -> f32 {
    let a = line_end.x - line_start.x;
    let b = line_end.y - line_start.y;

    let c = a * (line_start.y - point.y) - (line_start.x - point.x) * b;
    let abs_c = c.abs();

    let root = (a * a + b * b).sqrt();

    if root == 0.0 {
        return f32::MAX;
    }

    abs_c / root
}
