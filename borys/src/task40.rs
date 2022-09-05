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
    really_local_optimizations::StripeSwap,
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

    let new_field = gen_field_by_rects(&sorted_res, test_case);
    for x in 0..n {
        for y in 0..m {
            assert_eq!(new_field[x][y], need[x][y]);
        }
    }

    sorted_res
}

pub fn task40(
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

    let mut best_stripe = StripeSwap {
        cnt_lines: 129,
        first_line: 0,
        second_line: 161,
        by_x: false,
    };
    for _ in 0..1 {
        best_stripe = best_stripe.rotate_cw(test_case);
    }
    dbg!(best_stripe);

    assert!(test_case.regions.len() == 1);
    let new_testcase = TestCase {
        start_field: best_stripe.apply_field(&test_case.start_field),
        expected: best_stripe.apply_field(&test_case.expected),
        test_id: test_case.test_id,
        regions: test_case.regions.clone(),
    };

    save_image(&new_testcase.expected, "../images/task40.expected.png");

    let mut new_rects = rects;

    let mut last_exp_score = FMAX;
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
