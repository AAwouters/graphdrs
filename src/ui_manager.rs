use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use crate::graph::parse_graph6_string;
use crate::graph_drawer::DrawConfig;
use crate::graph_interface::GraphInterface;
use crate::svg_writer::draw_graph_to_file;
use crate::Content;

pub const UI_WIDTH: f32 = 300.0;

pub struct UIData {
    pub g6_string: String,
    pub highlight_g6_string: String,
    pub keep_embedding: bool,
    pub apply_force: bool,
    pub align_to_square_grid: bool,
    pub align_to_circular_grid: bool,
    pub grid_size: f32,
    pub svg_file_name: String,
    pub draw_config: DrawConfig,
}

impl UIData {
    pub fn new() -> Self {
        Self {
            g6_string: String::new(),
            highlight_g6_string: String::new(),
            keep_embedding: false,
            apply_force: false,
            align_to_square_grid: false,
            align_to_circular_grid: false,
            grid_size: 30.0,
            svg_file_name: String::new(),
            draw_config: DrawConfig::default(),
        }
    }
}

pub fn handle_ui(content: &mut Content) {
    let data = &mut content.ui_data;

    let id = hash!();

    widgets::Window::new(
        id,
        vec2(main_screen_width(), 0.0),
        vec2(UI_WIDTH, screen_height()),
    )
    .label("Settings")
    .titlebar(true)
    .movable(false)
    .ui(&mut root_ui(), |ui| {
        ui.tree_node(hash!(), "controls", |ui| {
            if ui.button(None, "Reset embedding") {
                content.embedding = GraphInterface::new(&content.graph);
            };
            ui.checkbox(hash!(), "Apply force", &mut data.apply_force);
            ui.checkbox(
                hash!(),
                "Align to square grid",
                &mut data.align_to_square_grid,
            );
            ui.checkbox(
                hash!(),
                "Align to circular grid",
                &mut data.align_to_circular_grid,
            );
            ui.slider(hash!(), "grid size", 10.0..50.0, &mut data.grid_size);
        });
        ui.tree_node(hash!(), "graph input", |ui| {
            ui.label(None, "Graph g6 string:");
            ui.input_text(hash!(), "", &mut data.g6_string);
            if ui.button(None, "Import graph") {
                if let Ok(graph) = parse_graph6_string(&data.g6_string) {
                    if !data.keep_embedding {
                        content.embedding = GraphInterface::new(&graph);
                    } else {
                        content.embedding.update_edges(&graph);
                    }
                    content.graph = graph;
                    data.g6_string = "".to_string();
                } else {
                    debug!("Error in parsing g6 graph");
                }
            }

            ui.checkbox(hash!(), "Keep vertex positions", &mut data.keep_embedding);
        });
        ui.tree_node(hash!(), "graph output", |ui| {
            ui.label(None, "SVG output file");
            ui.input_text(hash!(), "", &mut data.svg_file_name);
            if ui.button(None, "Export to SVG") {
                draw_graph_to_file(&content.drawable_graph, &data.svg_file_name)
                    .unwrap_or_else(|error| error!("{}", error));
            }
        });
        ui.tree_node(hash!(), "draw config", |ui| {
            ui.label(None, "Highlight g6 string:");
            ui.input_text(hash!(), "", &mut data.highlight_g6_string);
            if ui.button(None, "Highlight edges from graph") {
                parse_and_add_highlighting(&mut data.highlight_g6_string, &mut content.embedding);
            }

            ui.separator();

            ui.label(None, "Highlighting history:");
            ui.label(
                None,
                &format!(
                    "Current: {:?}, History size: {}",
                    content.embedding.current_highlight_graph,
                    content.embedding.get_history_size()
                ),
            );

            if ui.button(None, "Next highlighting") {
                content.embedding.set_next_highlighting();
            }
            if ui.button(None, "Previous highlighting") {
                content.embedding.set_previous_highlighting();
            }

            if ui.button(None, "Clear edge highlighting") {
                content.embedding.clear_edge_highlighting();
            }

            if ui.button(None, "Clear highlighting history") {
                content.embedding.clear_highlight_history();
            }

            ui.checkbox(
                hash!(),
                "draw vertex index",
                &mut data.draw_config.vertex_config.draw_index,
            );
            if data.draw_config.vertex_config.draw_index {
                ui.checkbox(
                    hash!(),
                    "zero-indexed vertices",
                    &mut data.draw_config.vertex_config.zero_indexed,
                );
            }
            ui.checkbox(
                hash!(),
                "draw edge index",
                &mut data.draw_config.edge_config.draw_index,
            );
            if data.draw_config.edge_config.draw_index {
                ui.checkbox(
                    hash!(),
                    "zero-indexed edges",
                    &mut data.draw_config.edge_config.zero_indexed,
                );
            }
        });
    });

    root_ui().move_window(id, Vec2::new(main_screen_width(), 0.0));
}

pub fn main_screen_width() -> f32 {
    screen_width() - UI_WIDTH
}

fn parse_and_add_highlighting(highlighting_string: &mut String, embedding: &mut GraphInterface) {
    let graphs = highlighting_string.lines();

    for graph in graphs {
        if let Ok(graph) = parse_graph6_string(graph) {
            embedding.set_edge_highlighting_and_add_to_history(graph);
        }
    }

    *highlighting_string = "".to_string();
}
