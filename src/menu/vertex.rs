use super::font::{ATLAS_HEIGHT, ATLAS_WIDTH, CELL_HEIGHT, CELL_WIDTH, Cell};
use super::geometry::Rect;
use super::primitive::Primitive;

pub const VERTICES_PER_PRIMITIVE: usize = 6;

const SOLID: f32 = 0.0;
const GLYPH: f32 = 1.0;
const CORNERS: [(f32, f32); VERTICES_PER_PRIMITIVE] = [
    (0.0, 0.0),
    (1.0, 0.0),
    (0.0, 1.0),
    (1.0, 0.0),
    (1.0, 1.0),
    (0.0, 1.0),
];
const WHOLE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 1.0,
    height: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position_uv: [f32; 4],
    pub color: [f32; 4],
    pub shape: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

pub fn build(primitives: &[Primitive], viewport: Viewport) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(primitives.len() * VERTICES_PER_PRIMITIVE);
    for primitive in primitives {
        push(&mut vertices, primitive, viewport);
    }
    vertices
}

fn push(into: &mut Vec<Vertex>, primitive: &Primitive, viewport: Viewport) {
    let (rect, color, kind, uv) = match primitive {
        Primitive::Rectangle { rect, color } => (*rect, *color, SOLID, WHOLE),
        Primitive::Glyph { rect, cell, color } => (*rect, *color, GLYPH, uv_of(*cell)),
    };
    for (horizontal, vertical) in CORNERS {
        into.push(Vertex {
            position_uv: [
                (rect.x + rect.width * horizontal) / viewport.width * 2.0 - 1.0,
                1.0 - (rect.y + rect.height * vertical) / viewport.height * 2.0,
                uv.x + uv.width * horizontal,
                uv.y + uv.height * vertical,
            ],
            color: color.components(),
            shape: [kind, 0.0, 0.0, 0.0],
        });
    }
}

fn uv_of(cell: Cell) -> Rect {
    Rect {
        x: (cell.column * CELL_WIDTH) as f32 / ATLAS_WIDTH as f32,
        y: (cell.row * CELL_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
        width: CELL_WIDTH as f32 / ATLAS_WIDTH as f32,
        height: CELL_HEIGHT as f32 / ATLAS_HEIGHT as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::{GLYPH, SOLID, VERTICES_PER_PRIMITIVE, Vertex, Viewport, build};
    use crate::menu::font::Cell;
    use crate::menu::geometry::Rect;
    use crate::menu::palette;
    use crate::menu::primitive::Primitive;

    fn viewport() -> Viewport {
        Viewport {
            width: 1920.0,
            height: 1080.0,
        }
    }

    fn full_screen() -> Primitive {
        Primitive::Rectangle {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
            color: palette::PANEL,
        }
    }

    fn glyph() -> Primitive {
        Primitive::Glyph {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 12.0,
                height: 16.0,
            },
            cell: Cell { column: 1, row: 2 },
            color: palette::VALUE,
        }
    }

    fn positions(vertices: &[Vertex]) -> Vec<(f32, f32)> {
        vertices
            .iter()
            .map(|vertex| (vertex.position_uv[0], vertex.position_uv[1]))
            .collect()
    }

    #[test]
    fn each_primitive_becomes_two_triangles() {
        let vertices = build(&[full_screen(), glyph()], viewport());

        assert_eq!(vertices.len(), 2 * VERTICES_PER_PRIMITIVE);
    }

    #[test]
    fn a_full_screen_rectangle_covers_the_whole_clip_space() {
        let vertices = build(&[full_screen()], viewport());
        let corners = positions(&vertices);

        assert!(corners.contains(&(-1.0, 1.0)));
        assert!(corners.contains(&(1.0, 1.0)));
        assert!(corners.contains(&(-1.0, -1.0)));
        assert!(corners.contains(&(1.0, -1.0)));
    }

    #[test]
    fn the_first_triangle_winds_clockwise_on_screen() {
        let vertices = build(&[full_screen()], viewport());
        let corners = positions(&vertices);

        assert_eq!(corners[0], (-1.0, 1.0));
        assert_eq!(corners[1], (1.0, 1.0));
        assert_eq!(corners[2], (-1.0, -1.0));
    }

    #[test]
    fn rectangles_and_glyphs_carry_different_kinds() {
        let vertices = build(&[full_screen(), glyph()], viewport());

        assert_eq!(vertices[0].shape[0], SOLID);
        assert_eq!(vertices[VERTICES_PER_PRIMITIVE].shape[0], GLYPH);
    }

    #[test]
    fn a_glyph_samples_only_its_own_atlas_cell() {
        let vertices = build(&[glyph()], viewport());
        let uvs: Vec<(f32, f32)> = vertices
            .iter()
            .map(|vertex| (vertex.position_uv[2], vertex.position_uv[3]))
            .collect();

        assert!(uvs.contains(&(6.0 / 96.0, 16.0 / 48.0)));
        assert!(uvs.contains(&(12.0 / 96.0, 24.0 / 48.0)));
    }

    #[test]
    fn a_rectangle_samples_the_whole_unit_square() {
        let vertices = build(&[full_screen()], viewport());

        assert_eq!(vertices[0].position_uv[2], 0.0);
        assert_eq!(vertices[4].position_uv[2], 1.0);
    }

    #[test]
    fn the_color_reaches_every_vertex() {
        let vertices = build(&[full_screen()], viewport());

        for vertex in &vertices {
            assert_eq!(vertex.color, palette::PANEL.components());
        }
    }

    #[test]
    fn the_vertex_matches_the_shader_stride() {
        assert_eq!(size_of::<Vertex>(), 48);
    }
}
