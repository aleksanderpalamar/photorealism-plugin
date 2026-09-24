use super::font_glyphs::{FIRST_CHARACTER, GLYPH_COUNT, GLYPH_HEIGHT, GLYPH_WIDTH, GLYPHS};

pub const CELL_WIDTH: usize = 6;
pub const CELL_HEIGHT: usize = 8;
pub const ATLAS_COLUMNS: usize = 16;
pub const ATLAS_ROWS: usize = 6;
pub const ATLAS_WIDTH: usize = ATLAS_COLUMNS * CELL_WIDTH;
pub const ATLAS_HEIGHT: usize = ATLAS_ROWS * CELL_HEIGHT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub column: usize,
    pub row: usize,
}

pub fn index_of(character: char) -> Option<usize> {
    let first = FIRST_CHARACTER as u32;
    let code = character as u32;
    let index = code.checked_sub(first)? as usize;
    (index < GLYPH_COUNT).then_some(index)
}

pub fn cell_of(character: char) -> Option<Cell> {
    let index = index_of(character)?;
    Some(Cell {
        column: index % ATLAS_COLUMNS,
        row: index / ATLAS_COLUMNS,
    })
}

pub fn text_width(text: &str) -> usize {
    text.chars().count() * CELL_WIDTH
}

pub fn atlas() -> Vec<u8> {
    let mut pixels = vec![0_u8; ATLAS_WIDTH * ATLAS_HEIGHT];
    for (index, glyph) in GLYPHS.iter().enumerate() {
        paint(&mut pixels, index, glyph);
    }
    pixels
}

fn paint(pixels: &mut [u8], index: usize, glyph: &[u8; GLYPH_WIDTH]) {
    let origin_x = (index % ATLAS_COLUMNS) * CELL_WIDTH;
    let origin_y = (index / ATLAS_COLUMNS) * CELL_HEIGHT;
    for (column, bits) in glyph.iter().enumerate() {
        for row in 0..GLYPH_HEIGHT {
            if bits & (1 << row) == 0 {
                continue;
            }
            pixels[(origin_y + row) * ATLAS_WIDTH + origin_x + column] = u8::MAX;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ATLAS_HEIGHT, ATLAS_WIDTH, CELL_HEIGHT, CELL_WIDTH, Cell, atlas, cell_of, index_of,
        text_width,
    };

    #[test]
    fn maps_the_printable_range_to_glyph_indexes() {
        assert_eq!(index_of(' '), Some(0));
        assert_eq!(index_of('A'), Some(33));
        assert_eq!(index_of('~'), Some(94));
    }

    #[test]
    fn rejects_characters_outside_the_printable_range() {
        assert_eq!(index_of('\n'), None);
        assert_eq!(index_of('\u{7f}'), None);
        assert_eq!(index_of('á'), None);
    }

    #[test]
    fn places_each_glyph_in_its_atlas_cell() {
        assert_eq!(cell_of(' '), Some(Cell { column: 0, row: 0 }));
        assert_eq!(cell_of('A'), Some(Cell { column: 1, row: 2 }));
        assert_eq!(cell_of('~'), Some(Cell { column: 14, row: 5 }));
    }

    #[test]
    fn the_atlas_covers_every_cell() {
        assert_eq!(atlas().len(), ATLAS_WIDTH * ATLAS_HEIGHT);
    }

    #[test]
    fn the_space_glyph_leaves_its_cell_empty() {
        let pixels = atlas();

        for row in 0..7 {
            for column in 0..5 {
                assert_eq!(pixels[row * ATLAS_WIDTH + column], 0);
            }
        }
    }

    #[test]
    fn the_letter_a_is_painted_from_its_column_bits() {
        let pixels = atlas();
        let origin = 2 * CELL_HEIGHT * ATLAS_WIDTH + CELL_WIDTH;

        assert_eq!(pixels[origin], 0);
        assert_eq!(pixels[origin + ATLAS_WIDTH], u8::MAX);
        assert_eq!(pixels[origin + 1], u8::MAX);
        assert_eq!(pixels[origin + ATLAS_WIDTH + 1], 0);
    }

    #[test]
    fn text_width_follows_the_cell_advance() {
        assert_eq!(text_width(""), 0);
        assert_eq!(text_width("abc"), 18);
    }
}
