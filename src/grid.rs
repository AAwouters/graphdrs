use macroquad::{
    prelude::{Vec2, WHITE},
    shapes::{draw_circle_lines, draw_line},
    window::*,
};

use crate::{graph_drawer::Drawable, ui_manager::main_screen_width};

pub struct SquareGrid {
    pub x_delta: f32,
    pub y_delta: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl SquareGrid {
    pub fn new(x_delta: f32, y_delta: f32) -> Self {
        Self {
            x_delta,
            y_delta,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    pub fn make_square(&mut self) {
        let min = self.x_delta.min(self.y_delta);
        self.x_delta = min;
        self.y_delta = min;
    }

    pub fn set_deltas(&mut self, x_delta: f32, y_delta: f32) {
        self.x_delta = x_delta;
        self.y_delta = y_delta;
    }

    pub fn set_deltas_square(&mut self, delta: f32) {
        self.set_deltas(delta, delta);
    }

    pub fn set_offsets_from_window(&mut self, window_dimensions: Vec2) {
        let mid = window_dimensions / 2.0;
        self.x_offset = mid.x % self.x_delta;
        self.y_offset = mid.y % self.y_delta;
    }

    pub fn delta_avg(&self) -> f32 {
        (self.x_delta + self.y_delta) * 0.5
    }
}

impl Drawable for SquareGrid {
    fn draw(&self) {
        let mut x = self.x_offset;
        let mut y = self.y_offset;

        while x < main_screen_width() {
            draw_line(x, 0.0, x, screen_height(), 2.0, WHITE);
            x += self.x_delta;
        }

        while y < screen_height() {
            draw_line(0.0, y, screen_width(), y, 2.0, WHITE);
            y += self.y_delta;
        }
    }
}

pub struct CircleGrid {
    pub r_delta: f32,
    pub center: Vec2,
    pub max: f32,
}

impl CircleGrid {
    pub fn new(r_delta: f32, window_dimensions: Vec2) -> Self {
        let max = window_dimensions.x.max(window_dimensions.y);
        let center = window_dimensions / 2.0;

        Self {
            r_delta,
            center,
            max,
        }
    }

    pub fn set_r_delta(&mut self, r_delta: f32) {
        self.r_delta = r_delta;
    }

    pub fn set_from_window(&mut self, window_dimensions: Vec2) {
        self.max = window_dimensions.x.max(window_dimensions.y);
        self.center = window_dimensions / 2.0;
    }
}

impl Drawable for CircleGrid {
    fn draw(&self) {
        let mut r = self.r_delta;

        while r < self.max {
            draw_circle_lines(self.center.x, self.center.y, r, 2.0, WHITE);
            r += self.r_delta;
        }
    }
}
