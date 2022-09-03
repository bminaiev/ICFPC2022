use std::{
    cmp::{max, min},
    collections::HashMap,
};

use algo_lib::{collections::array_2d::Array2D, misc::float_min_max::fmax};

use crate::{
    color::Color,
    consts::{COLOR_COST, LINE_CUT_COST, MERGE_COST, POINT_CUT_COST},
    op::Op,
    rect_id::{rect_id_from_usize, rect_id_sub_key},
    test_case::{Rect, TestCase},
    Point,
};

pub struct ApplyOpsResult {
    pub picture: Array2D<Color>,
    pub only_ops_cost: f64,
}

pub fn gen_start_field(test_case: &TestCase) -> Array2D<Color> {
    let n = test_case.expected.len();
    let m = test_case.expected[0].len();
    let mut a = Array2D::new(Color::default(), n, m);
    for region in test_case.regions.iter() {
        for x in region.rect.from.x..region.rect.to.x {
            for y in region.rect.from.y..region.rect.to.y {
                a[x as usize][y as usize] = region.color;
            }
        }
    }
    a
}

pub fn apply_ops(ops: &[Op], test_case: &TestCase) -> ApplyOpsResult {
    let n = test_case.expected.len();
    let m = test_case.expected[0].len();

    let canvas_size = (n as f64) * (m as f64);
    let mut a = gen_start_field(test_case);

    let mut rects = HashMap::new();
    for region in test_case.regions.iter() {
        rects.insert(region.name.clone(), region.rect);
    }
    let mut cost = 0.0;

    let mut last_rect_id = test_case.regions.len() - 1;
    for op in ops.iter() {
        match op {
            Op::CutPoint(id, p) => {
                let r = *rects.get(id).unwrap();
                let x1 = r.from.x;
                let x2 = p.x;
                let x3 = r.to.x;
                let y1 = r.from.y;
                let y2 = p.y;
                let y3 = r.to.y;

                rects.insert(
                    rect_id_sub_key(id, 0),
                    Rect::new(Point::new(x1, y1), Point::new(x2, y2)),
                );
                rects.insert(
                    rect_id_sub_key(id, 1),
                    Rect::new(Point::new(x2, y1), Point::new(x3, y2)),
                );
                rects.insert(
                    rect_id_sub_key(id, 2),
                    Rect::new(Point::new(x2, y2), Point::new(x3, y3)),
                );
                rects.insert(
                    rect_id_sub_key(id, 3),
                    Rect::new(Point::new(x1, y2), Point::new(x2, y3)),
                );
                rects.remove(id);
                cost += (POINT_CUT_COST * canvas_size / r.size()).round();
            }
            Op::Color(id, color) => {
                let r = *rects.get(id).unwrap();
                for x in r.from.x..r.to.x {
                    for y in r.from.y..r.to.y {
                        a[x as usize][y as usize] = *color;
                    }
                }
                cost += (COLOR_COST * canvas_size / r.size()).round();
            }
            Op::CutY(id, split_y) => {
                let r = *rects.get(id).unwrap();
                rects.insert(
                    rect_id_sub_key(id, 0),
                    Rect::new(r.from, Point::new(r.to.x, *split_y)),
                );
                rects.insert(
                    rect_id_sub_key(id, 1),
                    Rect::new(Point::new(r.from.x, *split_y), r.to),
                );
                rects.remove(id);
                cost += (LINE_CUT_COST * canvas_size / r.size()).round();
            }
            Op::CutX(id, split_x) => {
                let r = *rects.get(id).unwrap();
                rects.insert(
                    rect_id_sub_key(id, 0),
                    Rect::new(r.from, Point::new(*split_x, r.to.y)),
                );
                rects.insert(
                    rect_id_sub_key(id, 1),
                    Rect::new(Point::new(*split_x, r.from.y), r.to),
                );
                rects.remove(id);
                cost += (LINE_CUT_COST * canvas_size / r.size()).round();
            }
            Op::Merge(id1, id2) => {
                let r1 = *rects.get(id1).unwrap();
                let r2 = *rects.get(id2).unwrap();
                let fr = Point::new(min(r1.from.x, r2.from.x), min(r1.from.y, r2.from.y));
                let to = Point::new(max(r1.to.x, r2.to.x), max(r1.to.y, r2.to.y));
                let new_r = Rect::new(fr, to);
                assert!(new_r.size() == r1.size() + r2.size());
                last_rect_id += 1;
                rects.insert(rect_id_from_usize(last_rect_id), new_r);
                cost += (MERGE_COST * canvas_size / fmax(r1.size(), r2.size())).round();
                rects.remove(id1);
                rects.remove(id2);
            }
        }
    }
    ApplyOpsResult {
        picture: a,
        only_ops_cost: cost,
    }
}
