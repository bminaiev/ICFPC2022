use algo_lib::{
    collections::{array_2d::Array2D, last_exn::LastExn},
    misc::rand::Random,
};

use crate::{
    color::Color,
    color_corner::color_corner,
    color_picker::ColorPicker,
    consts::FMAX,
    interpreter::apply_ops,
    merger::merge,
    op::Op,
    ops_by_rects::gen_ops_by_solution_rects,
    pixel_dist::{
        estimate_pixel_distance_range_one_color, get_pixel_distance_range_one_color, EstimateResult,
    },
    test_case::TestCase,
    utils::p,
    Point,
};

#[derive(Clone, Copy)]
enum BestWay {
    FullColor(Color),
    SplitX(usize),
    SplitY(usize),
    SolveRec,
}

#[derive(Clone, Copy, Debug)]
pub struct SolutionRect {
    pub from: Point,
    pub to: Point,
    pub color: Color,
}

struct SmallerSolPts {
    x_map: Vec<usize>,
    y_map: Vec<usize>,
    sol: Box<Solution>,
}

struct Solution {
    dp: Array2D<Array2D<f64>>,
    dp_prev: Array2D<Array2D<BestWay>>,
    xs: Vec<usize>,
    ys: Vec<usize>,
    max_next_x_blocks: usize,
    max_next_y_blocks: usize,
    smaller_sol: Option<SmallerSolPts>,
}

impl Solution {
    pub fn iter_rects(
        &self,
        x_it: usize,
        y_it: usize,
        x_len: usize,
        y_len: usize,
        rects: &mut Vec<SolutionRect>,
    ) {
        let pr = self.dp_prev[x_it][y_it][x_len][y_len];
        match pr {
            BestWay::FullColor(color) => rects.push(SolutionRect {
                from: p(self.xs[x_it], self.ys[y_it]),
                to: p(self.xs[x_it + x_len], self.ys[y_it + y_len]),
                color,
            }),
            BestWay::SplitX(x_mid_it) => {
                self.iter_rects(x_it, y_it, x_mid_it - x_it, y_len, rects);
                self.iter_rects(x_mid_it, y_it, x_it + x_len - x_mid_it, y_len, rects);
            }
            BestWay::SplitY(y_mid_it) => {
                self.iter_rects(x_it, y_it, x_len, y_mid_it - y_it, rects);
                self.iter_rects(x_it, y_mid_it, x_len, y_it + y_len - y_mid_it, rects);
            }
            BestWay::SolveRec => {
                if let Some(smaller) = &self.smaller_sol {
                    let x_it2 = smaller.x_map[x_it + x_len];
                    let x_it = smaller.x_map[x_it];
                    let y_it2 = smaller.y_map[y_it + y_len];
                    let y_it = smaller.y_map[y_it];
                    // dbg!("go rec!", smaller.sol.max_next_x_blocks);
                    smaller
                        .sol
                        .iter_rects(x_it, y_it, x_it2 - x_it, y_it2 - y_it, rects);
                } else {
                    unreachable!();
                }
            }
        }
    }
}

fn get_mapping(xs: &[usize], smaller_xs: &[usize]) -> Vec<usize> {
    let mut res = vec![];
    for x in xs.iter() {
        res.push(smaller_xs.binary_search(x).unwrap())
    }
    res
}

fn find_solution(
    expected: &Array2D<Color>,
    xs: &[usize],
    ys: &[usize],
    color_picker: &ColorPicker,
    max_next_x_blocks: usize,
    max_next_y_blocks: usize,
    smaller_sol: Option<Box<Solution>>,
    rnd: &mut Random,
) -> Solution {
    let n = expected.len();
    let m = expected[0].len();

    let x_map = match &smaller_sol {
        None => vec![],
        Some(sol) => get_mapping(xs, &sol.xs),
    };
    let y_map = match &smaller_sol {
        None => vec![],
        Some(sol) => get_mapping(ys, &sol.ys),
    };

    let simple_elem = Array2D::new(FMAX, max_next_x_blocks + 1, max_next_y_blocks + 1);
    let simple_elem_prev = Array2D::new(
        BestWay::FullColor(Color::default()),
        max_next_x_blocks + 1,
        max_next_y_blocks + 1,
    );
    let mut dp = Array2D::new(simple_elem.clone(), xs.len() - 1, ys.len() - 1);
    let mut dp_prev = Array2D::new(simple_elem_prev.clone(), xs.len() - 1, ys.len() - 1);
    for x_it in (0..xs.len() - 1).rev() {
        for y_it in (0..ys.len() - 1).rev() {
            let x_start = xs[x_it];
            let y_start = ys[y_it];
            let mut cur_dp = simple_elem.clone();
            let mut cur_dp_prev = simple_elem_prev.clone();
            for x_len in 1..cur_dp.len() {
                for y_len in 1..cur_dp[x_len].len() {
                    let x2_it = x_it + x_len;
                    let y2_it = y_it + y_len;
                    if x2_it >= xs.len() || y2_it >= ys.len() {
                        continue;
                    }
                    let x_end = xs[x2_it];
                    let y_end = ys[y2_it];

                    if let Some(smaller_sol) = &smaller_sol {
                        // recursively
                        let x_it_smaller = x_map[x_it];
                        let y_it_smaller = y_map[y_it];
                        let x2_it_smaller = x_map[x2_it];
                        let y2_it_smaller = y_map[y2_it];
                        let x_len_smaller = x2_it_smaller - x_it_smaller;
                        let y_len_smaller = y2_it_smaller - y_it_smaller;
                        if x_len_smaller <= smaller_sol.max_next_x_blocks
                            && y_len_smaller <= smaller_sol.max_next_y_blocks
                        {
                            let cost = smaller_sol.dp[x_it_smaller][y_it_smaller][x_len_smaller]
                                [y_len_smaller];
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::SolveRec;
                            }
                        }
                    }

                    {
                        // split by x
                        for x_mid_it in x_it + 1..x2_it {
                            let cost = cur_dp[x_mid_it - x_it][y_len]
                                + dp[x_mid_it][y_it][x2_it - x_mid_it][y_len];
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::SplitX(x_mid_it);
                            }
                        }
                    }
                    {
                        // split by y
                        for y_mid_it in y_it + 1..y2_it {
                            let cost = cur_dp[x_len][y_mid_it - y_it]
                                + dp[x_it][y_mid_it][x_len][y2_it - y_mid_it];
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::SplitY(y_mid_it);
                            }
                        }
                    }

                    {
                        // color fully
                        let best_color =
                            color_picker.pick_color(p(x_start, y_start), p(x_end, y_end));
                        let cost_to_color =
                            color_corner(n, m, p(x_start, y_start), 0, best_color).cost;
                        let estimated_pixel_dist = estimate_pixel_distance_range_one_color(
                            best_color,
                            expected,
                            x_start..x_end,
                            y_start..y_end,
                            rnd,
                        );
                        const ESTIMATE_COEF: f64 = 0.8;
                        let estimated_value = match estimated_pixel_dist {
                            EstimateResult::RealResult(x) => x,
                            EstimateResult::Estimation(x) => x,
                        };
                        let coef = match estimated_pixel_dist {
                            EstimateResult::RealResult(_) => 1.0,
                            EstimateResult::Estimation(_) => ESTIMATE_COEF,
                        };
                        if estimated_value * coef + cost_to_color < cur_dp[x_len][y_len] {
                            let mut cost = cost_to_color;
                            match estimated_pixel_dist {
                                EstimateResult::RealResult(x) => cost += x,
                                EstimateResult::Estimation(_) => {
                                    cost += get_pixel_distance_range_one_color(
                                        best_color,
                                        expected,
                                        x_start..x_end,
                                        y_start..y_end,
                                    )
                                }
                            };
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::FullColor(best_color);
                            }
                        }
                    }
                }
            }

            dp[x_it][y_it] = cur_dp;
            dp_prev[x_it][y_it] = cur_dp_prev;
        }
    }
    let smaller_sol = match smaller_sol {
        Some(smaller_sol) => Some(SmallerSolPts {
            sol: smaller_sol,
            x_map,
            y_map,
        }),
        None => None,
    };
    Solution {
        dp,
        dp_prev,
        xs: xs.to_vec(),
        ys: ys.to_vec(),
        max_next_x_blocks,
        max_next_y_blocks,
        smaller_sol,
    }
}

fn add_more_coords(coords: &[usize], rnd: &mut Random) -> Vec<usize> {
    let mut res = vec![coords[0]];
    const BETWEEN: usize = 2;
    for i in 0..(coords.len() - 1) {
        let prev = coords[i];
        let next = coords[i + 1];
        let cnt_inside = next - prev - 1;
        if cnt_inside <= BETWEEN {
            for j in prev + 1..next {
                res.push(j);
            }
        } else {
            let mut used = vec![false; cnt_inside];
            let mut added = 0;
            while added != BETWEEN {
                let pos = rnd.gen(0..cnt_inside);
                if used[pos] {
                    continue;
                }
                used[pos] = true;
                added += 1;
            }
            for j in 0..used.len() {
                if used[j] {
                    res.push(prev + 1 + j);
                }
            }
        }
        res.push(next);
    }
    res
}

pub struct SolutionRes {
    pub a: Array2D<Color>,
    pub expected_score: f64,
    pub ops: Vec<Op>,
}

fn gen_coords(block_size: usize, max_value: usize) -> Vec<usize> {
    let mut res = vec![0];
    loop {
        let last = *res.last_exn();
        if last + block_size >= max_value {
            res.push(max_value);
            break;
        } else {
            res.push(last + block_size);
        }
    }
    res
}

pub fn solve_one(test_case: &TestCase, block_size: usize, use_third_layer: bool) -> SolutionRes {
    let expected = &test_case.expected;
    let color_picker = ColorPicker::new(&expected);
    let n = expected.len();
    let m = expected[0].len();
    // const BLOCK_SIZE: usize = 6;
    let xs = gen_coords(block_size, n);
    let ys = gen_coords(block_size, m);
    let blocks_n = xs.len() - 1;
    let blocks_m = ys.len() - 1;

    let mut rnd = Random::new(7877881);
    let more_xs = add_more_coords(&xs, &mut rnd);
    let more_ys = add_more_coords(&ys, &mut rnd);

    let more_xs_2 = add_more_coords(&more_xs, &mut rnd);
    let more_ys_2 = add_more_coords(&more_ys, &mut rnd);
    // dbg!(&xs);
    // dbg!(&more_xs);
    // dbg!(&more_xs_2);

    const MAX_NEXT_BLOCKS: usize = 15;

    let smallest_solution = if use_third_layer {
        Some(Box::new(find_solution(
            expected,
            &more_xs_2,
            &more_ys_2,
            &color_picker,
            MAX_NEXT_BLOCKS,
            MAX_NEXT_BLOCKS,
            None,
            &mut rnd,
        )))
    } else {
        None
    };

    let small_solution = find_solution(
        expected,
        &more_xs,
        &more_ys,
        &color_picker,
        MAX_NEXT_BLOCKS,
        MAX_NEXT_BLOCKS,
        smallest_solution,
        &mut rnd,
    );
    // dbg!("small solution done!");
    let solution = find_solution(
        expected,
        &xs,
        &ys,
        &color_picker,
        blocks_n,
        blocks_m,
        Some(Box::new(small_solution)),
        &mut rnd,
    );

    let mut rects = vec![];
    solution.iter_rects(0, 0, xs.len() - 1, ys.len() - 1, &mut rects);

    dbg!(rects.len());

    let merge_result = merge(&test_case);
    let after_merge_ops_applied = apply_ops(&merge_result.ops, test_case);
    let merge_cost = after_merge_ops_applied.only_ops_cost;
    let rect_ops: Vec<Op> = gen_ops_by_solution_rects(&rects, n, m, merge_result.last_block_id);
    let mut all_ops = merge_result.ops;
    all_ops.extend(rect_ops);
    // for r in rects.iter() {
    //     dbg!(
    //         (r.to.x - r.from.x) as usize / BLOCK_SIZE,
    //         (r.to.y - r.from.y) as usize / BLOCK_SIZE
    //     );
    // }

    let mut my_picture = Array2D::new(Color::START, n, m);
    for r in rects.iter() {
        let c = r.color; //Color::new_rand(&mut rnd);
        for x in r.from.x..r.to.x {
            for y in r.from.y..r.to.y {
                my_picture[x as usize][y as usize] = c;
            }
        }
    }

    let ops_res = apply_ops(&all_ops, test_case);
    for i in 0..n {
        for j in 0..m {
            assert_eq!(my_picture[i][j], ops_res.picture[i][j]);
        }
    }

    let score_without_merge = solution.dp[0][0][xs.len() - 1][ys.len() - 1];
    dbg!(score_without_merge);
    dbg!(score_without_merge + merge_cost);
    SolutionRes {
        a: my_picture,
        expected_score: score_without_merge + merge_cost,
        ops: all_ops,
    }
}
