#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    pub fn fraction_at(self, x: f32) -> f32 {
        if self.width <= 0.0 {
            return 0.0;
        }
        ((x - self.x) / self.width).clamp(0.0, 1.0)
    }

    pub fn centered_row(self, height: f32) -> Self {
        Self {
            x: self.x,
            y: self.y + (self.height - height) / 2.0,
            width: self.width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Rect};

    fn sample() -> Rect {
        Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        }
    }

    #[test]
    fn reports_its_far_edges() {
        assert_eq!(sample().right(), 110.0);
        assert_eq!(sample().bottom(), 60.0);
    }

    #[test]
    fn centering_keeps_the_horizontal_extent() {
        let centered = sample().centered_row(10.0);

        assert_eq!(centered.x, sample().x);
        assert_eq!(centered.width, sample().width);
        assert_eq!(centered.height, 10.0);
    }

    #[test]
    fn centering_leaves_equal_margins() {
        let centered = sample().centered_row(10.0);

        assert_eq!(
            centered.y - sample().y,
            sample().bottom() - centered.bottom()
        );
    }

    #[test]
    fn a_row_taller_than_its_parent_overflows_symmetrically() {
        let centered = sample().centered_row(60.0);

        assert_eq!(centered.y, 10.0);
    }

    #[test]
    fn contains_covers_the_edges_but_not_the_outside() {
        let rect = sample();

        assert!(rect.contains(Point { x: 10.0, y: 20.0 }));
        assert!(rect.contains(Point { x: 110.0, y: 60.0 }));
        assert!(rect.contains(Point { x: 60.0, y: 40.0 }));
        assert!(!rect.contains(Point { x: 9.0, y: 40.0 }));
        assert!(!rect.contains(Point { x: 60.0, y: 61.0 }));
    }

    #[test]
    fn a_horizontal_position_becomes_a_fraction() {
        let rect = sample();

        assert_eq!(rect.fraction_at(10.0), 0.0);
        assert_eq!(rect.fraction_at(110.0), 1.0);
        assert_eq!(rect.fraction_at(60.0), 0.5);
    }

    #[test]
    fn a_position_outside_the_rectangle_saturates() {
        let rect = sample();

        assert_eq!(rect.fraction_at(-100.0), 0.0);
        assert_eq!(rect.fraction_at(1000.0), 1.0);
    }
}
