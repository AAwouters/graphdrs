use macroquad::{
    prelude::{Color, Vec2},
    window::screen_height,
};
use std::{io::Write, path::Path};
use thiserror::Error;

use crate::{
    graph_drawer::{DrawableEdge, DrawableGraph, DrawableLabel, DrawableVertex},
    ui_manager::main_screen_width,
};

const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#;
const DOCSTRING: &str = r#"<!-- Created with GraphDrs -->"#;

pub type SVGOperationResult = Result<(), SVGWriterError>;

pub struct SVGWriter {
    svg_string: String,
    indentation_level: usize,
    indentation_size: usize,
    has_header: bool,
    finalised: bool,
}

impl SVGWriter {
    pub fn new() -> Self {
        let svg_string = String::new();

        Self {
            svg_string,
            indentation_level: 0,
            indentation_size: 4,
            has_header: false,
            finalised: false,
        }
    }

    pub fn write_header(&mut self, width: f32, height: f32) -> SVGOperationResult {
        if self.has_header {
            return Err(SVGWriterError::AlreadyHasHeader);
        }

        self.svg_string.push_str(XML_HEADER);
        self.svg_string.push('\n');
        self.svg_string.push_str(DOCSTRING);
        self.svg_string.push('\n');
        self.svg_string.push('\n');

        self.svg_string.push_str("<svg\n");
        self.indentation_level += 1;

        self.has_header = true;

        self.add_item(&SVGViewBox { width, height })?;
        self.add_item(&r#"version="1.1""#.to_string())?;
        self.add_item(&r#"xmlns="http://www.w3.org/2000/svg">"#.to_string())?;

        Ok(())
    }

    pub fn finalise(&mut self) -> SVGOperationResult {
        if !self.has_header {
            return Err(SVGWriterError::MissingHeader);
        }

        if self.finalised {
            return Err(SVGWriterError::AlreadyFinalised);
        }

        if self.indentation_level != 1 {
            return Err(SVGWriterError::UnexpectedIndentationLevel {
                expected: 1,
                found: self.indentation_level,
            });
        }

        self.indentation_level -= 1;

        self.svg_string.push_str("</svg>\n");

        self.finalised = true;

        Ok(())
    }

    pub fn add_item<T: SVGItem>(&mut self, item: &T) -> SVGOperationResult {
        if !self.has_header {
            return Err(SVGWriterError::MissingHeader);
        }

        if self.finalised {
            return Err(SVGWriterError::AlreadyFinalised);
        }

        for line in item.to_svg_string().lines() {
            self.svg_string.push_str(&format!(
                "{:2$}{}\n",
                "",
                line,
                self.indentation_level * self.indentation_size
            ));
        }

        Ok(())
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> SVGOperationResult {
        if !self.finalised {
            return Err(SVGWriterError::NotFinalised);
        }

        let mut file = std::fs::File::create(path)
            .map_err(|error| SVGWriterError::FileIOError { source: error })?;

        file.write_all(self.svg_string.as_bytes())
            .map_err(|error| SVGWriterError::FileIOError { source: error })?;

        Ok(())
    }
}

pub fn draw_graph_to_file<P: AsRef<Path>>(graph: &DrawableGraph, path: P) -> SVGOperationResult {
    let mut writer = SVGWriter::new();

    writer.write_header(main_screen_width(), screen_height())?;
    writer.add_item(graph)?;
    writer.finalise()?;

    writer.write_to_file(path)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum SVGWriterError {
    #[error("Header was not yet created")]
    MissingHeader,
    #[error("Header already created")]
    AlreadyHasHeader,
    #[error("SVG data already finalised")]
    AlreadyFinalised,
    #[error("SVG data not yet finalised")]
    NotFinalised,
    #[error("Unexpected indentation level. Expected: {expected}, found: {found}")]
    UnexpectedIndentationLevel { expected: usize, found: usize },
    #[error("Error in file IO: {source}")]
    FileIOError {
        #[from]
        source: std::io::Error,
    },
}

pub trait SVGItem {
    fn to_svg_string(&self) -> String;
}

impl SVGItem for String {
    fn to_svg_string(&self) -> String {
        self.to_owned()
    }
}

impl SVGItem for Color {
    fn to_svg_string(&self) -> String {
        let bytes: [u8; 4] = (*self).into();

        format!("#{:02X}{:02X}{:02X}", bytes[0], bytes[1], bytes[2])
    }
}

impl SVGItem for DrawableLabel {
    fn to_svg_string(&self) -> String {
        let mut string = String::new();

        string.push_str("<text");

        string.push_str(&format!(
            r#" x="{}" y="{}""#,
            self.position.x, self.position.y
        ));

        string.push_str(&format!(
            r#" fill="{}" font-size="24""#,
            self.color.to_svg_string()
        ));

        string.push('>');
        string.push_str(&self.content);
        string.push_str("</text>\n");
        string
    }
}

fn svg_circle(position: Vec2, radius: f32, color: Color) -> String {
    let mut string = String::new();

    string.push_str("<circle");

    string.push_str(&format!(
        r#" cx="{}" cy="{}" r="{}""#,
        position.x, position.y, radius
    ));

    string.push_str(&format!(r#" fill="{}""#, color.to_svg_string()));

    string.push_str("/>\n");
    string
}

impl SVGItem for DrawableVertex {
    fn to_svg_string(&self) -> String {
        let mut string = String::new();

        string.push_str(&svg_circle(
            self.position,
            self.border_radius,
            self.border_color,
        ));

        string.push_str(&svg_circle(
            self.position,
            self.main_radius,
            self.main_color,
        ));

        if let Some(label) = &self.label {
            string.push_str(&label.to_svg_string());
        }

        string
    }
}

impl SVGItem for DrawableEdge {
    fn to_svg_string(&self) -> String {
        let mut string = String::new();

        string.push_str("<line");

        string.push_str(&format!(r#" x1="{}" y1="{}""#, self.start.x, self.start.y));
        string.push_str(&format!(r#" x2="{}" y2="{}""#, self.end.x, self.end.y));
        string.push_str(&format!(
            r#" stroke="{}" stroke-width="{}""#,
            self.color.to_svg_string(),
            self.width
        ));

        string.push_str("/>");
        string.push('\n');

        if let Some(label) = &self.label {
            string.push_str(&label.to_svg_string());
        }

        string
    }
}

struct SVGViewBox {
    width: f32,
    height: f32,
}

impl SVGItem for SVGViewBox {
    fn to_svg_string(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!(r#"viewBox="0 0 {} {}""#, self.width, self.height,));

        string
    }
}

impl SVGItem for DrawableGraph {
    fn to_svg_string(&self) -> String {
        let mut string = String::new();

        for edge in &self.edges {
            string.push_str(&edge.to_svg_string());
        }

        for vertex in &self.vertices {
            string.push_str(&vertex.to_svg_string());
        }

        string
    }
}

#[cfg(test)]
mod tests {
    use macroquad::{color::*, prelude::Vec2};

    use super::*;

    #[test]
    fn test_black() {
        let black = BLACK;
        assert_eq!(black.to_svg_string(), "#000000");
    }

    #[test]
    fn test_white() {
        let white = WHITE;
        assert_eq!(white.to_svg_string(), "#FFFFFF");
    }

    #[test]
    fn test_red() {
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(red.to_svg_string(), "#FF0000");
    }

    #[test]
    fn test_green() {
        let green = Color::new(0.0, 1.0, 0.0, 1.0);
        assert_eq!(green.to_svg_string(), "#00FF00");
    }

    #[test]
    fn test_blue() {
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blue.to_svg_string(), "#0000FF");
    }

    #[test]
    fn test_label() {
        let label = DrawableLabel {
            content: "test label".to_string(),
            position: Vec2::new(0.0, 0.0),
            size: 10.0,
            color: WHITE,
        };

        let string = r##"<text x="0" y="0" fill="#FFFFFF">test label</text>"##.to_string();
        print!("printed: {}", &string);

        assert_eq!(label.to_svg_string(), string);
    }
}
