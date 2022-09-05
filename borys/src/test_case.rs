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

#[derive(Clone)]
pub struct Region {
    pub name: RectId,
    pub rect: Rect,
    pub color: Color,
}

#[derive(Clone)]
pub struct TestCase {
    pub expected: Array2D<Color>,
    pub start_field: Array2D<Color>,
    pub regions: Vec<Region>,
    pub test_id: usize,
}

impl TestCase {
    pub fn get_size(&self) -> (usize, usize) {
        (self.expected.len(), self.expected[0].len())
    }

    pub fn get_line_cut_cost(&self) -> f64 {
        if self.test_id <= 35 || self.test_id > 40 {
            7.0
        } else {
            2.0
        }
    }

    pub fn get_point_cut_cost(&self) -> f64 {
        if self.test_id <= 35 || self.test_id > 40 {
            10.0
        } else {
            3.0
        }
    }
}
