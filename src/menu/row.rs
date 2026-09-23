use super::field::Field;
use super::geometry::Rect;

const ROW_HEIGHT: f32 = 12.0;
const HANDLE_WIDTH: f32 = 5.0;
const HANDLE_HEIGHT: f32 = 9.0;
const SWITCH_SIZE: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub field: Field,
    pub bounds: Rect,
    pub label: Rect,
    pub track: Rect,
    pub value: Rect,
}

impl Row {
    pub fn handle(self, fraction: f32) -> Rect {
        let span = self.track.width - self.handle_width();
        Rect {
            x: self.track.x + span * fraction.clamp(0.0, 1.0),
            y: self.bounds.y + (self.bounds.height - self.handle_height()) / 2.0,
            width: self.handle_width(),
            height: self.handle_height(),
        }
    }

    pub fn slider_area(self) -> Rect {
        Rect {
            x: self.track.x,
            y: self.bounds.y,
            width: self.track.width,
            height: self.bounds.height,
        }
    }

    pub fn switch_box(self) -> Rect {
        let size = self.bounds.height * SWITCH_SIZE / ROW_HEIGHT;
        Rect {
            x: self.track.x,
            y: self.bounds.y + (self.bounds.height - size) / 2.0,
            width: size,
            height: size,
        }
    }

    fn handle_width(self) -> f32 {
        self.bounds.height * HANDLE_WIDTH / ROW_HEIGHT
    }

    fn handle_height(self) -> f32 {
        self.bounds.height * HANDLE_HEIGHT / ROW_HEIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, Rect, Row};

    fn row() -> Row {
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 24.0,
        };
        Row {
            field: Field::Exposure,
            bounds,
            label: bounds,
            track: Rect {
                x: 100.0,
                y: 10.0,
                width: 96.0,
                height: 6.0,
            },
            value: bounds,
        }
    }

    #[test]
    fn the_handle_is_centered_on_the_row() {
        let handle = row().handle(0.5);

        assert_eq!(
            handle.y - row().bounds.y,
            row().bounds.bottom() - handle.bottom()
        );
    }

    #[test]
    fn the_handle_scales_with_the_row_height() {
        let handle = row().handle(0.0);

        assert_eq!(handle.width, 10.0);
        assert_eq!(handle.height, 18.0);
    }

    #[test]
    fn the_slider_area_spans_the_track_and_the_row_height() {
        let area = row().slider_area();

        assert_eq!(area.x, row().track.x);
        assert_eq!(area.width, row().track.width);
        assert_eq!(area.height, row().bounds.height);
    }

    #[test]
    fn the_switch_box_is_square() {
        let box_rect = row().switch_box();

        assert_eq!(box_rect.width, box_rect.height);
        assert_eq!(box_rect.width, 16.0);
    }
}
