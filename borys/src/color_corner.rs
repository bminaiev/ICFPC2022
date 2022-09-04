use algo_lib::misc::float_min_max::{fmax, fmin};

use crate::{
    color::Color,
    consts::{COLOR_COST, MERGE_COST},
    op::Op,
    rect_id::{rect_id_from_usize, rect_id_sub_key},
    test_case::TestCase,
    Point,
};

pub struct ColorToCorner {
    pub cost: f64,
    pub ops: Vec<Op>,
    pub cur_whole_id: usize,
}

pub fn color_corner(
    n: usize,
    m: usize,
    p: Point,
    cur_whole_id: usize,
    color: Color,
    test_case: &TestCase,
) -> ColorToCorner {
    let main_id = rect_id_from_usize(cur_whole_id);
    let full_area = (n as f64) * (m as f64);

    let x_right = (n - p.x as usize) as f64;
    let y_up = (m - p.y as usize) as f64;
    let x_left = p.x as f64;
    let y_bottom = p.y as f64;

    let s0 = x_left * y_bottom;
    let s1 = x_right * y_bottom;
    let s2 = x_right * y_up;
    let s3 = x_left * y_up;

    if p.x == 0 || p.y == 0 {
        if p.x == 0 && p.y == 0 {
            return ColorToCorner {
                cost: COLOR_COST,
                ops: vec![Op::Color(rect_id_from_usize(cur_whole_id), color)],
                cur_whole_id,
            };
        }
        if p.x == 0 {
            let mut ops = vec![];
            ops.push(Op::CutY(main_id.clone(), p.y));
            ops.push(Op::Color(rect_id_sub_key(&main_id, 1), color));
            ops.push(Op::Merge(
                rect_id_sub_key(&main_id, 0),
                rect_id_sub_key(&main_id, 1),
            ));

            return ColorToCorner {
                cost: test_case.get_line_cut_cost()
                    + (COLOR_COST * full_area / s2).round()
                    + (MERGE_COST * full_area / fmax(s1, s2)).round(),
                ops,
                cur_whole_id: cur_whole_id + 1,
            };
        }
        assert!(p.y == 0);
        {
            let mut ops = vec![];
            ops.push(Op::CutX(main_id.clone(), p.x));
            ops.push(Op::Color(rect_id_sub_key(&main_id, 1), color));
            ops.push(Op::Merge(
                rect_id_sub_key(&main_id, 0),
                rect_id_sub_key(&main_id, 1),
            ));

            return ColorToCorner {
                cost: test_case.get_line_cut_cost()
                    + (COLOR_COST * full_area / s2).round()
                    + (MERGE_COST * full_area / fmax(s3, s2)).round(),
                ops,
                cur_whole_id: cur_whole_id + 1,
            };
        }
    }

    let mut ops = vec![];

    ops.push(Op::CutPoint(main_id.clone(), p));
    ops.push(Op::Color(rect_id_sub_key(&main_id, 2), color));
    let merge_y_first = (full_area / fmax(s0, s3) * MERGE_COST).round()
        + (full_area / fmax(s1, s2) * MERGE_COST).round()
        + (full_area / fmax(s0 + s3, s1 + s2) * MERGE_COST).round();
    let merge_x_first = (full_area / fmax(s0, s1) * MERGE_COST).round()
        + (full_area / fmax(s2, s3) * MERGE_COST).round()
        + (full_area / fmax(s0 + s1, s2 + s3) * MERGE_COST).round();
    if merge_x_first < merge_y_first {
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 0),
            rect_id_sub_key(&main_id, 1),
        ));
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 3),
            rect_id_sub_key(&main_id, 2),
        ));
    } else {
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 2),
            rect_id_sub_key(&main_id, 1),
        ));
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 3),
            rect_id_sub_key(&main_id, 0),
        ));
    }
    ops.push(Op::Merge(
        rect_id_from_usize(cur_whole_id + 1),
        rect_id_from_usize(cur_whole_id + 2),
    ));
    let merge_back = fmin(merge_y_first, merge_x_first);
    ColorToCorner {
        cost: test_case.get_point_cut_cost() + (COLOR_COST * full_area / s2).round() + merge_back,
        ops,
        cur_whole_id: cur_whole_id + 3,
    }
}
