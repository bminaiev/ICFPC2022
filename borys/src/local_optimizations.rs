use std::cmp::min;

use algo_lib::dbg;
use algo_lib::iters::shifts::SHIFTS_8;
use algo_lib::misc::rand::Random;
use algo_lib::misc::simulated_annealing::SearchFor;
use algo_lib::misc::simulated_annealing::SimulatedAnnealing;
use algo_lib::{collections::array_2d::Array2D, misc::min_max::UpdateMinMax};

use crate::color_corner::color_corner;
use crate::color_picker::estimate_best_color;
use crate::color_picker::find_best_color;
use crate::interpreter::gen_start_field;
use crate::merger::{merge, MergeResult};
use crate::ops_by_rects::gen_ops_by_solution_rects;
use crate::solver::SolutionRes;
use crate::test_case::TestCase;
use crate::utils::is_point_inside;
use crate::utils::p;
use crate::{
    color::Color, interpreter::apply_ops, op::Op, pixel_dist::get_pixel_distance,
    solver::SolutionRect,
};

#[inline(never)]
pub fn gen_field_by_rects(rects: &[SolutionRect], test_case: &TestCase) -> Array2D<Color> {
    let mut my = gen_start_field(test_case);
    let (n, m) = test_case.get_size();
    let by_ids = colored_by_rect(rects, n, m);
    for x in 0..n {
        for y in 0..m {
            if by_ids[x][y] != std::usize::MAX {
                my[x][y] = rects[by_ids[x][y]].color;
            }
        }
    }
    my
}

#[inline(never)]
pub fn colored_by_rect(rects: &[SolutionRect], n: usize, m: usize) -> Array2D<usize> {
    let mut res = Array2D::new(std::usize::MAX, n, m);
    for (r_id, r) in rects.iter().enumerate() {
        res[r.from.x as usize][r.from.y as usize] = r_id;
    }
    for x in 0..n {
        for y in 0..m {
            if res[x][y] != std::usize::MAX {
                if x + 1 < n && (res[x + 1][y] == std::usize::MAX || res[x + 1][y] < res[x][y]) {
                    res[x + 1][y] = res[x][y];
                }
                if y + 1 < m && (res[x][y + 1] == std::usize::MAX || res[x][y + 1] < res[x][y]) {
                    res[x][y + 1] = res[x][y];
                }
            }
        }
    }
    res
}

#[inline(never)]
pub fn get_covered_pixels_if_add_new_rect(
    old_colored: &Array2D<usize>,
    x_start: usize,
    y_start: usize,
    expected: &Array2D<Color>,
    my_id: usize,
) -> Vec<Color> {
    let n = expected.len();
    let m = expected[0].len();
    let mut res = vec![];
    for x in x_start..n {
        if old_colored[x][y_start] >= my_id {
            break;
        }
        for y in y_start..m {
            if old_colored[x][y] >= my_id {
                break;
            }
            res.push(expected[x][y]);
        }
    }

    res
}

pub fn shrink_rects(rects: &mut Vec<SolutionRect>, n: usize, m: usize) {
    let colored_by_rects = colored_by_rect(&rects, n, m);
    for r in rects.iter_mut() {
        r.to = r.from;
    }
    for x in 0..n {
        for y in 0..m {
            if colored_by_rects[x][y] == std::usize::MAX {
                continue;
            }
            let rect = &mut rects[colored_by_rects[x][y]];
            rect.to.x.update_max(x as i32 + 1);
            rect.to.y.update_max(y as i32 + 1);
        }
    }
}

pub fn optimize_colors(
    rects: &[SolutionRect],
    ops: &[Op],
    test_case: &TestCase,
    merge_result: &MergeResult,
) -> SolutionRes {
    let (n, m) = test_case.get_size();
    let expected = &test_case.expected;
    let final_res = apply_ops(ops, test_case);
    let pixel_dist = get_pixel_distance(&final_res.picture, &expected);

    let my = gen_field_by_rects(rects, test_case);
    for x in 0..n {
        for y in 0..m {
            assert_eq!(my[x][y], final_res.picture[x][y]);
        }
    }

    let colored_by_rect = colored_by_rect(rects, n, m);
    let mut pixels_by_rect = vec![vec![]; rects.len()];
    for x in 0..n {
        for y in 0..m {
            if colored_by_rect[x][y] == std::usize::MAX {
                continue;
            }
            pixels_by_rect[colored_by_rect[x][y]].push(expected[x][y]);
        }
    }
    let mut new_rects = rects.to_vec();
    for it in 0..pixels_by_rect.len() {
        let new_color = find_best_color(&pixels_by_rect[it], rects[it].color);
        new_rects[it].color = new_color;
    }
    let new_my = gen_field_by_rects(&new_rects, test_case);
    let new_pixel_dist = get_pixel_distance(&new_my, &expected);

    // dbg!(final_res.only_ops_cost + pixel_dist);
    // dbg!(final_res.only_ops_cost + new_pixel_dist);

    shrink_rects(&mut new_rects, n, m);
    let new_ops = gen_ops_by_solution_rects(&new_rects, n, m, merge_result, test_case);
    let final_res2 = apply_ops(&new_ops, test_case);
    let pixel_dist3 = get_pixel_distance(&final_res2.picture, &expected);
    // dbg!(final_res2.only_ops_cost + pixel_dist3);
    SolutionRes {
        ops: new_ops,
        a: final_res2.picture,
        expected_score: final_res2.only_ops_cost + pixel_dist3,
        last_block_id: final_res2.last_block_id.clone(),
    }
}

pub fn score_by_rects(rects: &[SolutionRect], test_case: &TestCase, merge_cost: f64) -> f64 {
    let my = gen_field_by_rects(&rects, test_case);
    let pixel_dist = get_pixel_distance(&my, &test_case.expected);
    let mut ops_cost = 0.0;
    let (n, m) = test_case.get_size();
    for r in rects.iter() {
        ops_cost += color_corner(n, m, r.from, 0, r.color, test_case).cost;
    }
    ops_cost + pixel_dist + merge_cost
}

pub fn optimize_positions(
    rects: &[SolutionRect],
    rnd: &mut Random,
    test_case: &TestCase,
) -> SolutionRes {
    let mut rects = rects.to_vec();
    let (n, m) = test_case.get_size();

    let merge_result = merge(test_case);
    let merge_cost = apply_ops(&merge_result.ops, test_case).only_ops_cost;

    let mut my_score = score_by_rects(&rects, test_case, merge_cost);
    let start_score = my_score;

    let mut not_changed_it = 0;
    // let start = Instant::now();
    let mut start_temp = rnd.gen_double() * 20.0;
    let finish_temp = 0.01;
    if start_temp <= finish_temp {
        start_temp = finish_temp * 2.0;
    }
    let mut sa = SimulatedAnnealing::new(
        20.0,
        SearchFor::MinimumScore,
        start_temp,
        finish_temp,
        my_score,
    );
    while sa.should_continue() {
        // if start.elapsed().as_secs_f64() > 10.0 {
        //     break;
        // }
        not_changed_it += 1;
        if not_changed_it == 1000 {
            break;
        }
        let rect_id = rnd.gen(0..rects.len());
        let change_type = rnd.gen(0..4);
        if change_type == 0 {
            let r = rects[rect_id];
            rects.remove(rect_id);
            let new_score = score_by_rects(&rects, test_case, merge_cost);
            if sa.should_go(new_score) {
                // dbg!("new best score! (by removing)", my_score, new_score);
                my_score = new_score;
                not_changed_it = 0;
            } else {
                rects.insert(rect_id, r);
                // assert!(score_by_rects(&rects, test_case, merge_cost) == my_score);
            }
        } else if change_type == 1 {
            let shift = SHIFTS_8[rnd.gen(0..4)];
            let prev_from = rects[rect_id].from;
            let new_from = rects[rect_id].from.apply_shift(&shift);

            if is_point_inside(new_from, n, m) {
                rects[rect_id].from = new_from;
                let new_score = score_by_rects(&rects, test_case, merge_cost);
                if sa.should_go(new_score) {
                    // dbg!("new best score!", my_score, new_score);
                    my_score = new_score;
                    not_changed_it = 0;
                } else {
                    rects[rect_id].from = prev_from;
                }
            }
        } else if change_type == 2 {
            // insert new rectangle
            if rnd.gen_double() < 0.5 {
                let new_from = p(rnd.gen(0..n), rnd.gen(0..m));

                let old_colored_by_rect = colored_by_rect(&rects, n, m);
                let old_id = old_colored_by_rect[new_from.x as usize][new_from.y as usize];
                if old_id == std::usize::MAX {
                    continue;
                }
                let idx = rnd.gen(old_id + 1..min(rects.len() + 1, old_id + 10)); //rnd.gen(0..rects.len() + 1);
                rects.insert(
                    idx,
                    SolutionRect {
                        from: new_from,
                        to: p(n, m),
                        color: Color::START,
                    },
                );

                let covered_pixels = get_covered_pixels_if_add_new_rect(
                    &old_colored_by_rect,
                    new_from.x as usize,
                    new_from.y as usize,
                    &test_case.expected,
                    idx,
                );

                let best_color =
                    find_best_color(&covered_pixels, estimate_best_color(&covered_pixels));
                rects[idx].color = best_color;
                let new_score = score_by_rects(&rects, test_case, merge_cost);
                if sa.should_go(new_score) {
                    // dbg!("new best score! (by adding new!!!)", my_score, new_score);
                    my_score = new_score;
                    not_changed_it = 0;
                } else {
                    rects.remove(idx);
                    // assert!(score_by_rects(&rects, test_case, merge_cost) == my_score);
                }
            }
        } else if change_type == 3 {
            if rects.len() > 1 {
                let pos = rnd.gen(0..rects.len() - 1);
                rects.swap(pos, pos + 1);
                let new_score = score_by_rects(&rects, test_case, merge_cost);
                if sa.should_go(new_score) {
                    // dbg!(
                    //     "new best score! (by swapping rects!!!)",
                    //     my_score,
                    //     new_score
                    // );
                    my_score = new_score;
                    not_changed_it = 0;
                } else {
                    rects.swap(pos, pos + 1);
                    // assert!(score_by_rects(&rects, test_case, merge_cost) == my_score);
                }
            }
        } else {
            unreachable!();
        }
    }

    dbg!(my_score);

    shrink_rects(&mut rects, n, m);
    let new_ops = gen_ops_by_solution_rects(&rects, n, m, &merge_result, test_case);
    let final_res2 = apply_ops(&new_ops, test_case);
    let pixel_dist3 = get_pixel_distance(&final_res2.picture, &test_case.expected);
    // dbg!(final_res2.only_ops_cost + pixel_dist3);
    let r = SolutionRes {
        ops: new_ops,
        a: final_res2.picture,
        expected_score: final_res2.only_ops_cost + pixel_dist3,
        last_block_id: final_res2.last_block_id,
    };
    dbg!("after local shift optimizations", r.expected_score);
    let after_local = optimize_colors(&rects, &r.ops, test_case, &merge_result);
    let diff = start_score - after_local.expected_score;
    dbg!(
        "after color optimizations",
        after_local.expected_score,
        start_score,
        diff
    );
    after_local
}
