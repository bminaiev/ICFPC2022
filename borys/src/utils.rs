use crate::Point;

pub fn p(x: usize, y: usize) -> Point {
    Point::new(x as i32, y as i32)
}

pub fn is_point_inside(p: Point, n: usize, m: usize) -> bool {
    p.x >= 0 && p.y >= 0 && (p.x as usize) < n && (p.y as usize) < m
}
