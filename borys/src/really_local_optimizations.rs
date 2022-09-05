use std::cmp::min;

use algo_lib::{
    collections::array_2d::Array2D,
    dbg,
    io::output::{output, set_global_output_to_file},
    iters::shifts::SHIFTS_8,
    misc::{
        rand::Random,
        simulated_annealing::{SearchFor, SimulatedAnnealing},
    },
    out, out_line,
};

use crate::{
    color::Color,
    color_corner::color_corner,
    color_picker::{estimate_best_color, find_best_color},
    consts::FMAX,
    interpreter::apply_ops,
    local_optimizations::{
        colored_by_rect, gen_field_by_rects, get_covered_pixels_if_add_new_rect, optimize_colors,
        optimize_positions, score_by_rects, shrink_rects,
    },
    merger::merge,
    ops_by_rects::{gen_ops_by_solution_rects, gen_rects_by_ops},
    pixel_dist::get_pixel_distance,
    rotator::{self, Rotator},
    savings::{save_image, save_solution},
    solver::{SolutionRect, SolutionRes},
    test_case::TestCase,
    utils::{is_point_inside, p},
    Point,
};

pub fn score_by_rects2(rects: &[SolutionRect], test_case: &TestCase, merge_cost: f64) -> f64 {
    let my = gen_field_by_rects(&rects, test_case);
    let pixel_dist = get_pixel_distance(&my, &test_case.expected);
    let mut ops_cost = 0.0;
    let (n, m) = test_case.get_size();
    for r in rects.iter() {
        ops_cost += color_corner(n, m, r.from, 0, r.color, test_case).cost;
    }
    dbg!(ops_cost, pixel_dist, ops_cost + pixel_dist);
    ops_cost + pixel_dist + merge_cost
}

#[derive(Clone, Copy, Debug)]
pub struct StripeSwap {
    pub cnt_lines: usize,
    pub first_line: usize,
    pub second_line: usize,
    pub by_x: bool,
}

impl StripeSwap {
    pub fn rotate_cw(&self, test_case: &TestCase) -> Self {
        let n = test_case.get_size().0;
        if self.by_x {
            Self {
                first_line: n - (self.second_line + self.cnt_lines),
                second_line: n - (self.first_line + self.cnt_lines),
                cnt_lines: self.cnt_lines,
                by_x: false,
            }
        } else {
            Self {
                first_line: self.first_line,
                second_line: self.second_line,
                cnt_lines: self.cnt_lines,
                by_x: true,
            }
        }
    }
}

impl StripeSwap {
    // 0 - nothing
    // 1 - first line
    // 2 - nothing
    // 3 - second line
    // 4 - nothing
    pub fn get_pt_type(&self, x: usize, y: usize) -> usize {
        if self.by_x {
            if x < self.first_line {
                return 0;
            }
            if x >= self.first_line && x < self.first_line + self.cnt_lines {
                return 1;
            }
            if x < self.second_line {
                return 2;
            }
            if x >= self.second_line && x < self.second_line + self.cnt_lines {
                return 3;
            }
            assert!(x > self.second_line + self.cnt_lines);
            return 4;
        } else {
            if y < self.first_line {
                return 0;
            }
            if y >= self.first_line && y < self.first_line + self.cnt_lines {
                return 1;
            }
            if y < self.second_line {
                return 2;
            }
            if y >= self.second_line && y < self.second_line + self.cnt_lines {
                return 3;
            }
            assert!(y > self.second_line + self.cnt_lines);
            return 4;
        }
    }

    pub fn apply_point(&self, x: usize, y: usize) -> (usize, usize) {
        if self.by_x {
            if x >= self.first_line && x < self.first_line + self.cnt_lines {
                return (self.second_line + x - self.first_line, y);
            }
            if x >= self.second_line && x < self.second_line + self.cnt_lines {
                return (self.first_line + x - self.second_line, y);
            }
            return (x, y);
        } else {
            if y >= self.first_line && y < self.first_line + self.cnt_lines {
                return (x, self.second_line + y - self.first_line);
            }
            if y >= self.second_line && y < self.second_line + self.cnt_lines {
                return (x, self.first_line + y - self.second_line);
            }
            return (x, y);
        }
    }

    pub fn apply_point_for_real(&self, cur_p: Point) -> Point {
        let (x, y) = self.apply_point(cur_p.x as usize, cur_p.y as usize);
        p(x, y)
    }

    pub fn apply_field(&self, a: &Array2D<Color>) -> Array2D<Color> {
        let n = a.len();
        let m = a[0].len();
        let mut res = a.clone();
        for i in 0..n {
            for j in 0..m {
                let (i2, j2) = self.apply_point(i, j);
                res[i2][j2] = a[i][j];
            }
        }
        res
    }
}

fn restore_rects_by_field(need: &Array2D<Color>, test_case: &TestCase) -> Vec<SolutionRect> {
    let mut res: Vec<SolutionRect> = vec![];
    const MX: usize = std::usize::MAX;
    let n = need.len();
    let m = need[0].len();
    let mut covered_by = Array2D::new(MX, n, m);
    let mut after: Vec<Vec<bool>> = vec![];

    let add_edge = |after: &mut Vec<Vec<bool>>, x: usize, y: usize| {
        after[x][y] = true;
        // let n = after.len();
        // for i in 0..n {
        //     for j in 0..n {
        //         for k in 0..n {
        //             if after[j][i] && after[i][k] {
        //                 after[j][k] = true;
        //             }
        //         }
        //     }
        // }
    };

    // after[x][y] == true means [x] should go after [y]
    for x in 0..n {
        for y in 0..m {
            let mut prevs = vec![];
            if x > 0 {
                prevs.push(covered_by[x - 1][y]);
            }
            if y > 0 {
                prevs.push(covered_by[x][y - 1]);
            }
            prevs.dedup();
            let need_color = need[x][y];
            let mut need_new = prevs.len() == 0;
            if prevs.len() == 1 {
                if res[prevs[0]].color != need_color {
                    need_new = true;
                } else {
                    covered_by[x][y] = prevs[0];
                }
            }
            if prevs.len() == 2 {
                if res[prevs[0]].color != need_color && res[prevs[1]].color != need_color {
                    need_new = true;
                } else if res[prevs[0]].color != need_color && after[prevs[0]][prevs[1]] {
                    need_new = true;
                } else if res[prevs[1]].color != need_color && after[prevs[1]][prevs[0]] {
                    need_new = true;
                } else if res[prevs[0]].color != need_color {
                    add_edge(&mut after, prevs[1], prevs[0]);
                    covered_by[x][y] = prevs[1];
                } else if res[prevs[1]].color != need_color {
                    add_edge(&mut after, prevs[0], prevs[1]);
                    covered_by[x][y] = prevs[0];
                } else {
                    if after[prevs[0]][prevs[1]] {
                        covered_by[x][y] = prevs[0];
                    } else {
                        add_edge(&mut after, prevs[1], prevs[0]);
                        covered_by[x][y] = prevs[1];
                    }
                }
            }
            if need_new {
                res.push(SolutionRect {
                    from: p(x, y),
                    to: p(n, m),
                    color: need_color,
                });
                for i in 0..after.len() {
                    after[i].push(false);
                }
                after.push(vec![false; after.len() + 1]);
                let new_id = after.len() - 1;
                covered_by[x][y] = new_id;
                for &pr in prevs.iter() {
                    add_edge(&mut after, new_id, pr);
                }
            }
            assert_ne!(covered_by[x][y], MX);
        }
    }
    let mut sorted_res = vec![];
    let mut used = vec![false; res.len()];
    loop {
        let mut changed = false;
        for i in 0..used.len() {
            if !used[i] {
                let mut ok = true;
                for j in 0..used.len() {
                    if !used[j] && after[i][j] {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    used[i] = true;
                    sorted_res.push(res[i]);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    assert!(res.len() == sorted_res.len());
    dbg!(sorted_res.len());

    let new_field = gen_field_by_rects(&sorted_res, test_case);
    for x in 0..n {
        for y in 0..m {
            if new_field[x][y] != need[x][y] {
                dbg!(x, y);
                dbg!(covered_by[x][y]);
                dbg!(covered_by[x - 1][y]);
                dbg!(covered_by[x][y - 1]);
                dbg!(res[covered_by[x - 1][y]].color);
                dbg!(res[covered_by[x][y - 1]].color);
                dbg!(need[x][y]);
                dbg!(after[33][44]);
                dbg!(after[44][33]);
                let r1 = res[covered_by[x - 1][y]];
                let r2 = res[covered_by[x][y - 1]];
                let p1 = sorted_res.iter().position(|z| z == &r1).unwrap();
                let p2 = sorted_res.iter().position(|z| z == &r2).unwrap();
                dbg!(p1, p2);
                dbg!(sorted_res[67]);
                dbg!(sorted_res[68]);
            }
            assert_eq!(new_field[x][y], need[x][y]);
        }
    }

    sorted_res
}

// really, no jokes
pub fn really_optimize_positions(
    rects: &[SolutionRect],
    rnd: &mut Random,
    test_case: &TestCase,
    rotator: &Rotator,
    test_case_before_rot: &TestCase,
) -> SolutionRes {
    let mut rects = rects.to_vec();
    dbg!("start rects...", rects.len());

    let (n, m) = test_case.get_size();
    let corners_cost = Array2D::new_f(n, m, |x, y| {
        color_corner(n, m, p(x, y), 0, Color::default(), test_case).cost
    });

    let calc_cost_of_rects = |rects: &[SolutionRect]| -> f64 {
        let mut sum_ops = 0.0;
        for r in rects.iter() {
            sum_ops += corners_cost[r.from.x as usize][r.from.y as usize];
        }
        sum_ops
    };

    let sum_ops = calc_cost_of_rects(&rects);
    dbg!(sum_ops);

    let mut best_cost = sum_ops;
    let mut best_stripe = StripeSwap {
        cnt_lines: 0,
        first_line: 0,
        second_line: 0,
        by_x: true,
    };

    {
        // split by [y]
        for cnt_lines in 1..n {
            for first_line in 0..n {
                for second_line in first_line + cnt_lines..n {
                    if second_line + cnt_lines <= n {
                        let mut new_cost = 0.0;
                        for r in rects.iter() {
                            let mut y = r.from.y as usize;
                            if y >= first_line && y < first_line + cnt_lines {
                                y = second_line + (y - first_line);
                            } else if y >= second_line && y < second_line + cnt_lines {
                                y = first_line + (y - second_line);
                            }
                            new_cost += corners_cost[r.from.x as usize][y];
                        }
                        if new_cost < best_cost {
                            best_cost = new_cost;
                            best_stripe = StripeSwap {
                                cnt_lines,
                                first_line,
                                second_line,
                                by_x: false,
                            };
                        }
                    }
                }
            }
        }
        dbg!(best_stripe);
        dbg!(sum_ops - best_cost);
    }

    {
        // split by [x]

        for cnt_lines in 1..n {
            for first_line in 0..n {
                for second_line in first_line + cnt_lines..n {
                    if second_line + cnt_lines <= n {
                        let mut new_cost = 0.0;
                        for r in rects.iter() {
                            let mut x = r.from.x as usize;
                            if x >= first_line && x < first_line + cnt_lines {
                                x = second_line + (x - first_line);
                            } else if x >= second_line && x < second_line + cnt_lines {
                                x = first_line + (x - second_line);
                            }
                            new_cost += corners_cost[x][r.from.y as usize];
                        }
                        if new_cost < best_cost {
                            best_cost = new_cost;
                            best_stripe = StripeSwap {
                                cnt_lines,
                                first_line,
                                second_line,
                                by_x: true,
                            };
                        }
                    }
                }
            }
        }
    }

    dbg!(best_stripe);
    dbg!(sum_ops - best_cost);

    assert!(test_case.regions.len() == 1);
    let new_testcase = TestCase {
        start_field: best_stripe.apply_field(&test_case.start_field),
        expected: best_stripe.apply_field(&test_case.expected),
        test_id: test_case.test_id,
        regions: test_case.regions.clone(),
    };

    let my_current = gen_field_by_rects(&rects, test_case);
    let my_current_target = best_stripe.apply_field(&my_current);

    let mut new_rects = restore_rects_by_field(&my_current_target, test_case);
    let new_rects_cost = calc_cost_of_rects(&new_rects);
    dbg!(new_rects_cost);

    // let mut new_rects = vec![];
    // shrink_rects(&mut rects, n, m);
    // for (id, r) in rects.iter().enumerate() {
    //     let p1 = r.from;
    //     let p2 = r.to;
    //     let t1 = best_lines.get_pt_type(p1.x as usize, p1.y as usize);
    //     let t2 = best_lines.get_pt_type(p2.x as usize - 1, p2.y as usize - 1);
    //     if t1 == t2 {
    //         let ntype = if t1 == 1 {
    //             3
    //         } else if t1 == 3 {
    //             1
    //         } else {
    //             t1
    //         };
    //         let nfrom = best_lines.apply_point_for_real(r.from);
    //         dbg!(id, ntype, p1.x, nfrom.x);
    //         new_rects.push((
    //             ntype,
    //             id,
    //             SolutionRect {
    //                 color: r.color,
    //                 to: r.to,
    //                 from: nfrom,
    //             },
    //         ));
    //     }
    // }
    // new_rects.sort_by_key(|&(a, b, c)| (a, b));

    // let mut new_rects: Vec<_> = new_rects.into_iter().map(|(_, _, r)| r).collect();
    let mut last_exp_score = FMAX;
    save_image(&new_testcase.expected, "../images/last.expected.png");
    loop {
        let call_real_opts = optimize_positions(&new_rects, rnd, &new_testcase);
        dbg!(call_real_opts.expected_score, last_exp_score);
        if call_real_opts.expected_score < last_exp_score {
            new_rects = gen_rects_by_ops(&call_real_opts.ops, n, m);
            last_exp_score = call_real_opts.expected_score;
            save_image(&call_real_opts.a, "../images/last.res.png");

            {
                let mut new_sol = rotator.rotate_sol(call_real_opts);
                let mut rotated_stripe = best_stripe;
                for _ in 0..rotator.rotations_to_answer() {
                    rotated_stripe = rotated_stripe.rotate_cw(test_case);
                }
                new_sol.add_stripe_swap(&rotated_stripe, test_case);

                let final_seq_of_ops = new_sol.ops;
                let initial_test_case = rotator.get_initial_test_case();
                for x in 0..n {
                    for y in 0..n {
                        assert_eq!(
                            initial_test_case.expected[x][y],
                            test_case_before_rot.expected[x][y]
                        );
                    }
                }
                // let final_ops_res = apply_ops(&final_seq_of_ops, &initial_test_case);
                // let pixel_diff =
                //     get_pixel_distance(&final_ops_res.picture, &initial_test_case.expected);

                let new_sol = SolutionRes::new_from_ops(&initial_test_case, &final_seq_of_ops);
                dbg!(new_sol.expected_score);
                save_image(&new_sol.a, "../images/last.check.png");
                save_image(&initial_test_case.expected, "../images/last.exp.check.png");

                save_solution(100 + test_case.test_id, &new_sol);
            }
        }
    }

    const SZ: usize = 3;
    {
        // costs of corners
        set_global_output_to_file("costs.txt");
        let (n, m) = test_case.get_size();
        for x in 0..n {
            if x % SZ == 0 {
                for y in 0..m {
                    if y % SZ == 0 {
                        let cost = color_corner(n, m, p(x, y), 0, Color::default(), test_case).cost;
                        out!(format!("{:4} ", cost));
                    }
                }
                out_line!();
            }
        }
        output().flush();
    }

    {
        score_by_rects2(&rects, test_case, 0.0);
        // pixel dists
        let my = gen_field_by_rects(&rects, test_case);
        set_global_output_to_file("dist.txt");
        let (n, m) = test_case.get_size();
        for x in 0..n {
            if x % SZ == 0 {
                for y in 0..m {
                    if y % SZ == 0 {
                        let d = my[x][y].dist(&test_case.expected[x][y]);
                        out!(format!("{:4} ", d as i32));
                    }
                }
                out_line!();
            }
        }
        output().flush();
    }

    todo!("At some point I will write it..")
}
