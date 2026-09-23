#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn components(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

pub const PANEL: Color = Color::new(0.020, 0.020, 0.030, 0.93);
pub const TITLE_BAR: Color = Color::new(0.045, 0.043, 0.065, 0.97);
pub const TITLE_TEXT: Color = Color::new(0.85, 0.85, 0.92, 1.0);
pub const LABEL: Color = Color::new(0.72, 0.72, 0.80, 1.0);
pub const VALUE: Color = Color::new(0.90, 0.90, 0.96, 1.0);
pub const TRACK: Color = Color::new(0.10, 0.10, 0.14, 1.0);
pub const FILL: Color = Color::new(0.33, 0.26, 0.72, 1.0);
pub const HANDLE: Color = Color::new(0.55, 0.50, 0.92, 1.0);
pub const LOCKED: Color = Color::new(0.30, 0.30, 0.36, 1.0);

pub fn label(editable: bool) -> Color {
    if editable { LABEL } else { LOCKED }
}

pub fn value(editable: bool) -> Color {
    if editable { VALUE } else { LOCKED }
}

pub fn fill(editable: bool) -> Color {
    if editable { FILL } else { LOCKED }
}

pub fn handle(editable: bool) -> Color {
    if editable { HANDLE } else { LOCKED }
}

#[cfg(test)]
mod tests {
    use super::{FILL, HANDLE, LABEL, LOCKED, PANEL, TRACK, VALUE, label, value};

    #[test]
    fn an_editable_row_uses_its_own_colors() {
        assert_eq!(label(true), LABEL);
        assert_ne!(value(true), LOCKED);
    }

    #[test]
    fn a_locked_row_uses_the_locked_color() {
        assert_eq!(label(false), LOCKED);
        assert_eq!(value(false), LOCKED);
    }

    #[test]
    fn the_panel_is_translucent_and_the_marks_are_opaque() {
        let marks = [LABEL, VALUE, FILL, HANDLE, TRACK, LOCKED];

        for mark in marks {
            assert_eq!(mark.alpha, 1.0);
            assert!(PANEL.alpha < mark.alpha);
        }
    }

    #[test]
    fn components_follow_the_shader_order() {
        assert_eq!(
            FILL.components(),
            [FILL.red, FILL.green, FILL.blue, FILL.alpha]
        );
    }
}
