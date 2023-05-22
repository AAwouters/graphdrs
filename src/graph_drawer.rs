use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::graph_interface::{DrawState, GraphInterface};

pub trait Drawable {
    fn draw(&self);
}

pub struct DrawableGraph {
    pub vertices: Vec<DrawableVertex>,
    pub edges: Vec<DrawableEdge>,
}

impl DrawableGraph {
    pub fn compose(embedding: &GraphInterface, config: &DrawConfig) -> Self {
        let mut vertices = Vec::with_capacity(embedding.vertex_properties.len());
        let vertex_config = &config.vertex_config;

        for (index, vertex_properties) in embedding.vertex_properties.iter().enumerate().rev() {
            let position = vertex_properties.position;

            let mut main_radius = vertex_config.main_size;
            let mut border_radius = vertex_config.border_size + main_radius;

            let (mut main_color, mut border_color) = {
                match vertex_properties.draw_state {
                    DrawState::Default => (vertex_config.main_color, vertex_config.border_color),
                    DrawState::Highlighted => {
                        (vertex_config.highlight_color, vertex_config.highlight_color)
                    }
                    DrawState::Unhighlighted => (
                        vertex_config.unhighlight_color,
                        vertex_config.unhighlight_color,
                    ),
                    DrawState::Hidden => (
                        Color::new(0.0, 0.0, 0.0, 0.0),
                        Color::new(0.0, 0.0, 0.0, 0.0),
                    ),
                }
            };

            let mut is_interacted = false;

            if let Some(hovered) = embedding.hovered_vertex {
                if hovered == index {
                    main_radius += 2.0;
                    border_radius += 2.0;

                    main_color = vertex_config.highlight_color;
                    border_color = vertex_config.highlight_color;

                    is_interacted = true;
                }
            }

            if let Some(dragged) = embedding.dragged_vertex {
                if dragged == index {
                    main_color = vertex_config.drag_color;
                    border_color = vertex_config.drag_color;

                    is_interacted = true;
                }
            }

            if vertex_properties.draw_state == DrawState::Hidden && !is_interacted {
                continue;
            }

            let label = if vertex_config.draw_index {
                let character_width = vertex_config.label_size;
                let mut string_width = character_width;

                if index >= 10 {
                    string_width += character_width;
                }

                let x_offset = -string_width / 2.0 + 9.0;
                let y_offset = character_width / 2.0 - 10.0;

                Some(DrawableLabel {
                    content: if vertex_config.zero_indexed {
                        index.to_string()
                    } else {
                        (index + 1).to_string()
                    },
                    position: position + vec2(x_offset, y_offset),
                    size: vertex_config.label_size,
                    color: vertex_config.label_color,
                })
            } else {
                None
            };

            let composed_vertex = DrawableVertex {
                position,
                main_radius,
                border_radius,
                main_color,
                border_color,
                label,
            };

            vertices.push(composed_vertex);
        }

        let mut edges = Vec::new();
        let edge_config = &config.edge_config;

        for (index, edge) in embedding.edge_properties.iter().enumerate() {
            let verices = edge.vertices;
            let start = embedding.get_position(verices.0);
            let end = embedding.get_position(verices.1);

            let mut width = edge_config.width;

            let mut color = match edge.draw_state {
                crate::graph_interface::DrawState::Default => edge_config.color,
                crate::graph_interface::DrawState::Highlighted => edge_config.highlight_color,
                crate::graph_interface::DrawState::Unhighlighted => edge_config.unhighlight_color,
                crate::graph_interface::DrawState::Hidden => Color::new(0.0, 0.0, 0.0, 0.0),
            };

            let mut is_hovered = false;

            if let Some(hovered) = embedding.hovered_edge {
                if hovered == index {
                    width += 2.0;

                    color = edge_config.highlight_color;

                    is_hovered = true;
                }
            }

            if edge.draw_state == DrawState::Hidden && !is_hovered {
                continue;
            }

            let label = if edge_config.draw_index {
                let min_vertex = edge.vertices.0.min(edge.vertices.1);
                let max_vertex = edge.vertices.0.max(edge.vertices.1);

                let label_index = max_vertex * (max_vertex - 1) / 2 + min_vertex;

                let offset = {
                    let diff = end - start;
                    let angle = (diff.y.atan2(diff.x) + PI) % PI - 0.3;

                    let x_offset = 10.0;
                    let y_offset = -10.0;

                    if angle < PI / 4.0 {
                        vec2(-x_offset, y_offset)
                    } else if angle < PI / 2.0 {
                        vec2(x_offset, y_offset)
                    } else if angle < 3.0 * PI / 4.0 {
                        vec2(-x_offset, y_offset)
                    } else {
                        vec2(x_offset, y_offset)
                    }
                };

                Some(DrawableLabel {
                    content: if edge_config.zero_indexed {
                        label_index.to_string()
                    } else {
                        (label_index + 1).to_string()
                    },
                    position: (start + end) / 2.0 + offset,
                    size: edge_config.label_size,
                    color: edge_config.label_color,
                })
            } else {
                None
            };

            let composed_edge = DrawableEdge {
                start,
                end,
                width,
                color,
                label,
            };

            edges.push(composed_edge);
        }

        Self { vertices, edges }
    }
}

impl Drawable for DrawableGraph {
    fn draw(&self) {
        for edge in &self.edges {
            edge.draw();
        }

        for vertex in &self.vertices {
            vertex.draw();
        }
    }
}

pub struct DrawableVertex {
    pub position: Vec2,
    pub main_radius: f32,
    pub border_radius: f32,
    pub main_color: Color,
    pub border_color: Color,
    pub label: Option<DrawableLabel>,
}

impl DrawableVertex {}

impl Default for DrawableVertex {
    fn default() -> Self {
        let config = VertexDrawConfig::default();

        Self {
            position: Vec2::ZERO,
            main_radius: config.main_size,
            border_radius: config.main_size + config.border_size,
            main_color: config.main_color,
            border_color: config.border_color,
            label: None,
        }
    }
}

impl Drawable for DrawableVertex {
    fn draw(&self) {
        draw_circle(
            self.position.x,
            self.position.y,
            self.border_radius,
            self.border_color,
        );

        draw_circle(
            self.position.x,
            self.position.y,
            self.main_radius,
            self.main_color,
        );

        if let Some(label) = &self.label {
            label.draw();
        }
    }
}

pub struct DrawableEdge {
    pub start: Vec2,
    pub end: Vec2,
    pub width: f32,
    pub color: Color,
    pub label: Option<DrawableLabel>,
}

impl DrawableEdge {}

impl Default for DrawableEdge {
    fn default() -> Self {
        let config = EdgeDrawConfig::default();

        Self {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            width: config.width,
            color: config.color,
            label: None,
        }
    }
}

impl Drawable for DrawableEdge {
    fn draw(&self) {
        draw_line(
            self.start.x,
            self.start.y,
            self.end.x,
            self.end.y,
            self.width,
            self.color,
        );

        if let Some(label) = &self.label {
            label.draw();
        }
    }
}

pub struct DrawableLabel {
    pub content: String,
    pub position: Vec2,
    pub size: f32,
    pub color: Color,
}

impl Drawable for DrawableLabel {
    fn draw(&self) {
        draw_text(
            &self.content,
            self.position.x,
            self.position.y,
            self.size,
            self.color,
        );
    }
}

pub struct DrawConfig {
    pub vertex_config: VertexDrawConfig,
    pub edge_config: EdgeDrawConfig,
    pub background_color: Color,
}

impl Default for DrawConfig {
    fn default() -> Self {
        Self {
            vertex_config: Default::default(),
            edge_config: Default::default(),
            background_color: Color::new(0.91, 0.91, 0.91, 1.00),
        }
    }
}

pub struct VertexDrawConfig {
    pub main_color: Color,
    pub border_color: Color,
    pub main_size: f32,
    pub border_size: f32,
    pub highlight_color: Color,
    pub unhighlight_color: Color,
    pub drag_color: Color,
    pub draw_index: bool,
    pub zero_indexed: bool,
    pub label_color: Color,
    pub label_size: f32,
}

impl Default for VertexDrawConfig {
    fn default() -> Self {
        Self {
            main_color: SKYBLUE,
            border_color: DARKBLUE,
            main_size: 12.0,
            border_size: 5.0,
            highlight_color: LIME,
            unhighlight_color: MAROON,
            drag_color: DARKBLUE,
            draw_index: true,
            zero_indexed: false,
            label_color: BLACK,
            label_size: 35.0,
        }
    }
}

pub struct EdgeDrawConfig {
    pub width: f32,
    pub color: Color,
    pub highlight_color: Color,
    pub unhighlight_color: Color,
    pub draw_index: bool,
    pub zero_indexed: bool,
    pub label_color: Color,
    pub label_size: f32,
}

impl Default for EdgeDrawConfig {
    fn default() -> Self {
        Self {
            width: 5.0,
            color: BLACK,
            highlight_color: MAROON,
            unhighlight_color: LIGHTGRAY,
            draw_index: false,
            zero_indexed: false,
            label_color: BLUE,
            label_size: 40.0,
        }
    }
}
