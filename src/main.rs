use std::time::{Duration, Instant};

use graph::Graph;
use graph_drawer::{Drawable, DrawableGraph};
use graph_interface::GraphInterface;
use grid::{CircleGrid, SquareGrid};
use macroquad::prelude::*;
use ui_manager::{handle_ui, main_screen_width, UIData, UI_WIDTH};

mod graph;
mod graph_drawer;
mod graph_interface;
mod grid;
mod svg_writer;
mod ui_manager;

pub struct Content {
    graph: Graph,
    embedding: GraphInterface,
    drawable_graph: DrawableGraph,
    ui_data: UIData,
}

impl Content {
    fn new() -> Self {
        let mut graph = Graph::new(4);
        graph.edges.push((0, 1));
        graph.edges.push((1, 2));
        graph.edges.push((2, 3));
        graph.edges.push((0, 3));
        graph.edges.push((1, 3));

        let embedding = GraphInterface::new(&graph);
        let ui_data = UIData::new();
        let drawable_graph = DrawableGraph::compose(&embedding, &ui_data.draw_config);

        Self {
            graph,
            embedding,
            drawable_graph,
            ui_data,
        }
    }
}

#[macroquad::main("graphdrs")]
async fn main() {
    macroquad::telemetry::disable();

    let mut content = Content::new();
    let mut square_grid = SquareGrid::new(30.0, 30.0);
    square_grid.make_square();
    let mut circular_grid = CircleGrid::new(30.0, vec2(400.0, 400.0));

    loop {
        let frame_start = Instant::now();
        clear_background(content.ui_data.draw_config.background_color);

        draw_text(
            &get_fps().to_string(),
            main_screen_width() - 40.0,
            20.0,
            30.0,
            WHITE,
        );

        content.embedding.handle_mouse_input();

        if content.ui_data.apply_force {
            content.embedding.apply_force(&content.graph);
        }

        if content.ui_data.align_to_square_grid {
            square_grid.set_deltas_square(content.ui_data.grid_size);
            square_grid.set_offsets_from_window(vec2(screen_width() - UI_WIDTH, screen_height()));
            square_grid.draw();
            content.embedding.align_to_square_grid(&square_grid);
        }

        if content.ui_data.align_to_circular_grid {
            circular_grid.set_r_delta(content.ui_data.grid_size);
            circular_grid.set_from_window(vec2(screen_width() - UI_WIDTH, screen_height()));
            circular_grid.draw();
            content.embedding.align_to_circular_grid(&circular_grid);
        }

        handle_ui(&mut content);

        content.drawable_graph =
            DrawableGraph::compose(&content.embedding, &content.ui_data.draw_config);

        content.drawable_graph.draw();

        let frame_end = Instant::now();

        #[cfg(any(unix))]
        std::thread::sleep(Duration::from_micros(16666) - (frame_end - frame_start));

        next_frame().await
    }
}
