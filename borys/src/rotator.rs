use algo_lib::collections::array_2d::Array2D;
use algo_lib::dbg;

use crate::{
    color::Color,
    interpreter::apply_ops,
    op::Op,
    rect_id::{rect_id_from_vec, rect_id_to_vec},
    solver::SolutionRes,
    test_case::{Rect, Region, TestCase},
    Point,
};

pub struct Rotator {
    pub test_case: TestCase,
    pub ops: Vec<Op>,
    rotated: i32,
}

fn rotate_ops_cw(ops: &[Op], n: usize) -> Vec<Op> {
    let n = n as i32;
    let mut last_cut_op = Op::Merge(vec![], vec![]);
    let mut new_ops = vec![];

    let rotate_id_vec = |id_vec: &mut Vec<usize>| {
        let sz = id_vec.len() - 1;
        if id_vec[sz] == 0 {
            id_vec[sz] = 3;
        } else {
            id_vec[sz] -= 1;
        }
    };

    for op in ops.iter() {
        let converted_op = match op {
            Op::CutPoint(id, p) => {
                last_cut_op = op.clone();
                Op::CutPoint(id.clone(), Point::new(p.y, n - p.x))
            }
            Op::Color(id, color) => {
                let mut id_vec = rect_id_to_vec(id);
                let pos = id_vec.len() - 1;
                match last_cut_op {
                    Op::CutPoint(_, _) => {
                        rotate_id_vec(&mut id_vec);
                    }
                    Op::CutY(_, _) => {}
                    Op::CutX(_, _) => {
                        id_vec[pos] ^= 1;
                    }
                    Op::Merge(_, _) => {}
                    Op::Color(_, _) => {}
                };
                Op::Color(rect_id_from_vec(&id_vec), *color)
            }
            Op::CutY(id, y_pos) => {
                last_cut_op = op.clone();
                Op::CutX(id.clone(), *y_pos)
            }
            Op::CutX(id, x_pos) => {
                last_cut_op = op.clone();
                Op::CutY(id.clone(), n - x_pos)
            }
            Op::Merge(id1, id2) => {
                let mut id1_vec = rect_id_to_vec(id1);
                let mut id2_vec = rect_id_to_vec(id2);

                if let Op::CutPoint(_,_ ) = last_cut_op && id1_vec.len() != 1 {
                    rotate_id_vec(&mut id1_vec);
                    rotate_id_vec(&mut id2_vec);
                    Op::Merge(rect_id_from_vec(&id1_vec), rect_id_from_vec(&id2_vec))
                } else {
                    Op::Merge(id1.clone(), id2.clone())
                }
            }
        };
        new_ops.push(converted_op);
    }
    new_ops
}

fn rotate_tc_cw(test_case: &TestCase) -> TestCase {
    let (n, m) = test_case.get_size();
    assert_eq!(n, m);
    let mut new_expected = Array2D::new(Color::default(), m, n);
    for x in 0..n {
        for y in 0..m {
            new_expected[y][n - 1 - x] = test_case.expected[x][y];
        }
    }

    let rotate_point = |x: i32, y: i32| -> Point { Point::new(y, n as i32 - x) };

    let new_regions = test_case
        .regions
        .iter()
        .map(|r| {
            let fr = r.rect.from;
            let to = r.rect.to;
            let new_rect = Rect::new(rotate_point(to.x, fr.y), rotate_point(fr.x, to.y));
            Region {
                name: r.name.clone(),
                color: r.color,
                rect: new_rect,
            }
        })
        .collect();

    TestCase {
        expected: new_expected,
        regions: new_regions,
        test_id: test_case.test_id,
    }
}

impl Rotator {
    pub fn new(test_case: &TestCase, ops: &[Op]) -> Self {
        let mut test_case = test_case.clone();
        let mut ops = ops.to_vec();
        let (n, m) = test_case.get_size();
        assert!(n == m);
        for rotated in 0..4 {
            let apply_res = apply_ops(&ops, &test_case);
            if apply_res.only_colored_top_right {
                dbg!("found good rotation", rotated, ops.len());
                return Self {
                    test_case,
                    ops,
                    rotated,
                };
            }
            ops = rotate_ops_cw(&ops, n);
            test_case = rotate_tc_cw(&test_case);
        }
        unreachable!("Bad ops. Can't rotate :(");
    }

    pub(crate) fn rotate_sol(&self, before_rotation: SolutionRes) -> SolutionRes {
        let more_rotations = (4 - self.rotated) % 4;
        let mut ops = before_rotation.ops;
        let mut test_case = self.test_case.clone();
        for _ in 0..more_rotations {
            ops = rotate_ops_cw(&ops, test_case.get_size().0);
            test_case = rotate_tc_cw(&test_case);
        }
        SolutionRes::new_from_ops(&test_case, &ops)
    }
}
