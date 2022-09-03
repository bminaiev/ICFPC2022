use algo_lib::collections::array_2d::Array2D;

use crate::{color::Color, rect_id::RectId, Point};

#[derive(Clone, Copy)]
pub struct Rect {
    pub from: Point,
    pub to: Point,
}

impl Rect {
    pub fn new(from: Point, to: Point) -> Self {
        Self { from, to }
    }

    pub fn size(&self) -> f64 {
        let dx = (self.to.x - self.from.x) as f64;
        let dy = (self.to.y - self.from.y) as f64;
        dx * dy
    }
}

pub struct Region {
    pub name: RectId,
    pub rect: Rect,
    pub color: Color,
}
pub struct TestCase {
    pub expected: Array2D<Color>,
    pub regions: Vec<Region>,
}
