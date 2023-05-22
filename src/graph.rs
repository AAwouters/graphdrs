pub struct Graph {
    pub vertices: usize,
    pub edges: Vec<(usize, usize)>,
}

impl Graph {
    pub fn new(vertices: usize) -> Self {
        Graph {
            vertices,
            edges: Vec::new(),
        }
    }
}

pub fn parse_graph6_string(g6_string: &str) -> Result<Graph, Graph6ParseError> {
    let g6_bytes = g6_string.as_bytes();

    let vertices = graph6_number_of_vertices(g6_string)?;

    let mut start_index = 0;
    if g6_bytes[start_index] == b'>' {
        start_index += 10;
    }

    if vertices <= 62 {
        start_index += 1;
    } else if vertices <= 64 {
        start_index += 4;
    } else {
        return Err(Graph6ParseError::UnsupportedGraphSize { supported_size: 64 });
    }

    let mut graph = Graph::new(vertices);

    let mut current_vertex = 1;
    let mut current_neighbour = 0;

    let mut index = start_index;
    'outer: while g6_bytes.get(index).is_some() {
        let mut current_bit = 1 << 5;
        let current_bits = g6_bytes[index] - 63;

        while current_bit != 0 {
            if (current_bits & current_bit) != 0 {
                graph.edges.push((current_neighbour, current_vertex));
            }

            current_neighbour += 1;
            if current_neighbour >= current_vertex {
                current_vertex += 1;
                current_neighbour = 0;
            }

            if current_vertex >= vertices {
                break 'outer;
            }

            current_bit >>= 1;
        }

        index += 1;
    }

    Ok(graph)
}

fn graph6_number_of_vertices(g6_string: &str) -> Result<usize, Graph6ParseError> {
    let g6_bytes = g6_string.as_bytes();

    if g6_bytes.is_empty() {
        return Err(Graph6ParseError::EmptyString);
    }

    let start_char = g6_bytes[0];

    if !(63..=126).contains(&start_char) && start_char != b'>' {
        return Err(Graph6ParseError::InvalidStartCharacter(g6_bytes[0] as char));
    }

    let mut index = 0;

    if g6_bytes[index] == b'>' {
        index += 10;
        if g6_bytes.get(index).is_none() {
            return Err(Graph6ParseError::UnexpectedStringEnd);
        }
    }

    if g6_bytes[index] < 126 {
        Ok((g6_bytes[index] - 63) as usize)
    } else {
        Err(Graph6ParseError::UnsupportedGraphSize { supported_size: 62 })
    }
}

pub enum Graph6ParseError {
    EmptyString,
    InvalidStartCharacter(char),
    UnexpectedStringEnd,
    UnsupportedGraphSize { supported_size: u32 },
}
