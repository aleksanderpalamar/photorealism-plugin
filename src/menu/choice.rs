use super::geometry::{Point, Rect};
use super::row::Row;

const INSET_SHARE: f32 = 12.0;

pub struct Choice {
    pub closed: Rect,
    pub items: Vec<Rect>,
}

impl Choice {
    pub fn build(row: &Row, count: usize) -> Self {
        let inset = row.bounds.height / INSET_SHARE;
        let closed = Rect {
            x: row.track.x,
            y: row.bounds.y + inset,
            width: row.value.right() - row.track.x,
            height: row.bounds.height - inset * 2.0,
        };
        let items = (0..count)
            .map(|index| Rect {
                y: row.bounds.bottom() + row.bounds.height * index as f32 + inset,
                ..closed
            })
            .collect();
        Self { closed, items }
    }

    pub fn item_at(&self, point: Point) -> Option<usize> {
        self.items.iter().position(|item| item.contains(point))
    }

    pub fn list(&self) -> Rect {
        let Some(first) = self.items.first() else {
            return self.closed;
        };
        let bottom = self.items.last().unwrap_or(first).bottom();
        Rect {
            height: bottom - first.y,
            ..*first
        }
    }
}

#[cfg(test)]
#[path = "choice_tests.rs"]
mod tests;
