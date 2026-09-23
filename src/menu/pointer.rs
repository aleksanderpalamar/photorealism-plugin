use super::geometry::Point;
use super::vertex::Viewport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Button {
    #[default]
    Released,
    Pressed,
}

impl Button {
    pub fn from_pressed(pressed: bool) -> Self {
        if pressed {
            Self::Pressed
        } else {
            Self::Released
        }
    }

    pub fn is_pressed(self) -> bool {
        matches!(self, Self::Pressed)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Pointer {
    position: Point,
    button: Button,
}

impl Pointer {
    pub fn position(self) -> Point {
        self.position
    }

    pub fn button(self) -> Button {
        self.button
    }

    pub fn set_button(&mut self, button: Button) {
        self.button = button;
    }

    pub fn move_by(&mut self, horizontal: f32, vertical: f32, viewport: Viewport) {
        self.position = Point {
            x: (self.position.x + horizontal).clamp(0.0, viewport.width),
            y: (self.position.y + vertical).clamp(0.0, viewport.height),
        };
    }

    pub fn center(&mut self, viewport: Viewport) {
        self.position = Point {
            x: viewport.width / 2.0,
            y: viewport.height / 2.0,
        };
        self.button = Button::Released;
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, Pointer};
    use crate::menu::vertex::Viewport;

    fn viewport() -> Viewport {
        Viewport {
            width: 1920.0,
            height: 1080.0,
        }
    }

    #[test]
    fn deltas_accumulate() {
        let mut pointer = Pointer::default();

        pointer.move_by(30.0, 20.0, viewport());
        pointer.move_by(5.0, -4.0, viewport());

        assert_eq!(pointer.position().x, 35.0);
        assert_eq!(pointer.position().y, 16.0);
    }

    #[test]
    fn the_pointer_never_leaves_the_viewport() {
        let mut pointer = Pointer::default();

        pointer.move_by(-500.0, -500.0, viewport());
        assert_eq!(pointer.position().x, 0.0);
        assert_eq!(pointer.position().y, 0.0);

        pointer.move_by(9000.0, 9000.0, viewport());
        assert_eq!(pointer.position().x, 1920.0);
        assert_eq!(pointer.position().y, 1080.0);
    }

    #[test]
    fn centering_places_the_pointer_and_releases_the_button() {
        let mut pointer = Pointer::default();
        pointer.set_button(Button::Pressed);

        pointer.center(viewport());

        assert_eq!(pointer.position().x, 960.0);
        assert_eq!(pointer.position().y, 540.0);
        assert!(!pointer.button().is_pressed());
    }

    #[test]
    fn the_button_reflects_what_was_reported() {
        assert!(Button::from_pressed(true).is_pressed());
        assert!(!Button::from_pressed(false).is_pressed());
        assert_eq!(Button::default(), Button::Released);
    }
}
