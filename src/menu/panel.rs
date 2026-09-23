use super::draw_list;
use super::geometry::Point;
use super::layout::Layout;
use super::vertex::{self, Vertex, Viewport};
use crate::settings::Settings;

const MARGIN: f32 = 12.0;
const REFERENCE_HEIGHT: f32 = 540.0;
const MINIMUM_SCALE: f32 = 2.0;

pub fn scale_for(height: f32) -> f32 {
    (height / REFERENCE_HEIGHT).floor().max(MINIMUM_SCALE)
}

pub fn vertices(settings: &Settings, viewport: Viewport, title: &str) -> Vec<Vertex> {
    let scale = scale_for(viewport.height);
    let origin = Point {
        x: MARGIN * scale,
        y: MARGIN * scale,
    };
    let layout = Layout::build(origin, scale);
    vertex::build(&draw_list::build(&layout, settings, title), viewport)
}

#[cfg(test)]
mod tests {
    use super::{scale_for, vertices};
    use crate::menu::vertex::{VERTICES_PER_PRIMITIVE, Viewport};
    use crate::settings::Settings;

    fn viewport(width: f32, height: f32) -> Viewport {
        Viewport { width, height }
    }

    #[test]
    fn the_scale_follows_the_vertical_resolution() {
        assert_eq!(scale_for(1080.0), 2.0);
        assert_eq!(scale_for(1440.0), 2.0);
        assert_eq!(scale_for(2160.0), 4.0);
    }

    #[test]
    fn small_displays_keep_the_minimum_scale() {
        assert_eq!(scale_for(720.0), 2.0);
        assert_eq!(scale_for(480.0), 2.0);
    }

    #[test]
    fn the_panel_produces_whole_triangles() {
        let built = vertices(&Settings::default(), viewport(1920.0, 1080.0), "menu");

        assert!(!built.is_empty());
        assert_eq!(built.len() % VERTICES_PER_PRIMITIVE, 0);
    }

    #[test]
    fn the_panel_stays_inside_clip_space() {
        let built = vertices(&Settings::default(), viewport(1920.0, 1080.0), "menu");

        for vertex in &built {
            assert!(vertex.position_uv[0] >= -1.0 && vertex.position_uv[0] <= 1.0);
            assert!(vertex.position_uv[1] >= -1.0 && vertex.position_uv[1] <= 1.0);
        }
    }

    #[test]
    fn the_panel_sits_in_the_upper_left_corner() {
        let built = vertices(&Settings::default(), viewport(1920.0, 1080.0), "menu");

        assert!(built[0].position_uv[0] < -0.9);
        assert!(built[0].position_uv[1] > 0.9);
    }
}
