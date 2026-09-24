#[derive(Clone, Copy, Debug, PartialEq)]
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
    use super::Rect;

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
}
