use std::cmp::min;
use std::time::Instant;

use algo_lib::dbg;
use algo_lib::iters::shifts::SHIFTS_8;
use algo_lib::misc::rand::Random;
use algo_lib::{collections::array_2d::Array2D, misc::min_max::UpdateMinMax};

use crate::color_corner::color_corner;
use crate::ops_by_rects::gen_ops_by_solution_rects;
use crate::solver::SolutionRes;
use crate::test_case::TestCase;
use crate::utils::p;
use crate::{
    color::Color,
    interpreter::apply_ops,
    op::Op,
    pixel_dist::{get_pixel_distance, PIXEL_DIST_COEF},
    solver::SolutionRect,
};
use crate::{test_case, Point};

fn color_dist(colors: &[Color], my: Color) -> f64 {
    let mut res = 0.0;
    for c in colors.iter() {
        res += c.dist(&my);
    }
    res * PIXEL_DIST_COEF
}

fn find_best_color(colors: &[Color], mut my: Color) -> Color {
    let start_dist = color_dist(colors, my);
    let mut cur_best_dist = start_dist;
    loop {
        let mut changed = false;
        for it in 0..4 {
            for delta in [-1, 1].iter() {
                let cur_color = my.0[it] as i32;
                let check = cur_color + delta;
                if check >= 0 && check <= 255 {
                    let mut try_color = my.clone();
                    try_color.0[it] = check as u8;
                    let new_dist = color_dist(colors, try_color);
                    if new_dist < cur_best_dist {
                        cur_best_dist = new_dist;
                        changed = true;
                        my = try_color;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    // dbg!(start_dist, cur_best_dist);
    my
}

fn estimate_best_color(colors: &[Color]) -> Color {
    let mut sum = vec![0.0; 4];
    for c in colors.iter() {
        for i in 0..4 {
            sum[i] += c.0[i] as f64;
        }
    }
    let mut res = Color::default();
    for i in 0..4 {
        let x = (sum[i] / (colors.len() as f64)).round();
        assert!(x >= 0.0);
        assert!(x <= 255.0);
        res.0[i] = x as u8;
    }
    // dbg!(start_dist, cur_best_dist);
    res
}

fn gen_field_by_rects(rects: &[SolutionRect], n: usize, m: usize) -> Array2D<Color> {
    let mut my = Array2D::new(Color::START, n, m);
    for r in rects.iter() {
        for x in r.from.x as usize..n {
            for y in r.from.y as usize..m {
                my[x][y] = r.color;
            }
        }
    }
    my
}

fn colored_by_rect(rects: &[SolutionRect], n: usize, m: usize) -> Array2D<usize> {
    let mut colored_by_rect = Array2D::new(std::usize::MAX, n, m);
    for (r_id, r) in rects.iter().enumerate() {
        for x in r.from.x as usize..n {
            for y in r.from.y as usize..m {
                colored_by_rect[x][y] = r_id;
            }
        }
    }
    colored_by_rect
}

fn shrink_rects(rects: &mut Vec<SolutionRect>, n: usize, m: usize) {
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
    expected: &Array2D<Color>,
    rects: &[SolutionRect],
    ops: &[Op],
    start_whole_id: usize,
    test_case: &TestCase,
) -> SolutionRes {
    let n = expected.len();
    let m = expected[0].len();
    let final_res = apply_ops(ops, test_case);
    let pixel_dist = get_pixel_distance(&final_res.picture, &expected);

    let my = gen_field_by_rects(rects, n, m);
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
    let new_my = gen_field_by_rects(&new_rects, n, m);
    let new_pixel_dist = get_pixel_distance(&new_my, &expected);

    dbg!(final_res.only_ops_cost + pixel_dist);
    dbg!(final_res.only_ops_cost + new_pixel_dist);

    shrink_rects(&mut new_rects, n, m);
    let new_ops = gen_ops_by_solution_rects(&new_rects, n, m, start_whole_id);
    let final_res2 = apply_ops(&new_ops, test_case);
    let pixel_dist3 = get_pixel_distance(&final_res2.picture, &expected);
    // dbg!(final_res2.only_ops_cost + pixel_dist3);
    SolutionRes {
        ops: new_ops,
        a: final_res2.picture,
        expected_score: final_res2.only_ops_cost + pixel_dist3,
    }
}

fn score_by_rects(rects: &[SolutionRect], n: usize, m: usize, expected: &Array2D<Color>) -> f64 {
    let my = gen_field_by_rects(&rects, n, m);
    let pixel_dist = get_pixel_distance(&my, &expected);
    let mut ops_cost = 0.0;
    for r in rects.iter() {
        ops_cost += color_corner(n, m, r.from, 0, r.color).cost;
    }
    ops_cost + pixel_dist
}

fn is_point_inside(p: Point, n: usize, m: usize) -> bool {
    p.x >= 0 && p.y >= 0 && (p.x as usize) < n && (p.y as usize) < m
}

pub fn optimize_positions(
    expected: &Array2D<Color>,
    rects: &[SolutionRect],
    ops: &[Op],
    rnd: &mut Random,
    start_whole_id: usize,
    test_case: &TestCase,
) -> SolutionRes {
    let mut rects = rects.to_vec();
    let n = expected.len();
    let m = expected[0].len();

    let mut my_score = score_by_rects(&rects, n, m, expected);
    let start_score = my_score;

    let mut not_changed_it = 0;
    let start = Instant::now();
    loop {
        if start.elapsed().as_secs_f64() > 10.0 {
            break;
        }
        not_changed_it += 1;
        if not_changed_it == 1000 {
            break;
        }
        let rect_id = rnd.gen(0..rects.len());
        let change_type = rnd.gen(0..4);
        if change_type == 0 {
            let r = rects[rect_id];
            rects.remove(rect_id);
            let new_score = score_by_rects(&rects, n, m, expected);
            if new_score < my_score {
                dbg!("new best score! (by removing)", my_score, new_score);
                my_score = new_score;
                not_changed_it = 0;
            } else {
                rects.insert(rect_id, r);
                assert!(score_by_rects(&rects, n, m, expected) == my_score);
            }
        } else if change_type == 1 {
            let shift = SHIFTS_8[rnd.gen(0..4)];
            let prev_from = rects[rect_id].from;
            let new_from = rects[rect_id].from.apply_shift(&shift);

            if is_point_inside(new_from, n, m) {
                rects[rect_id].from = new_from;
                let new_score = score_by_rects(&rects, n, m, expected);
                if new_score < my_score {
                    dbg!("new best score!", my_score, new_score);
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
                let colored_by_rect = colored_by_rect(&rects, n, m);
                assert!(colored_by_rect[new_from.x as usize][new_from.y as usize] == idx);
                let mut covered_pixels = vec![];
                for x in 0..n {
                    for y in 0..m {
                        if colored_by_rect[x][y] == idx {
                            covered_pixels.push(expected[x][y]);
                        }
                    }
                }
                let best_color =
                    find_best_color(&covered_pixels, estimate_best_color(&covered_pixels));
                rects[idx].color = best_color;
                let new_score = score_by_rects(&rects, n, m, expected);
                if new_score < my_score {
                    dbg!("new best score! (by adding new!!!)", my_score, new_score);
                    my_score = new_score;
                    not_changed_it = 0;
                } else {
                    rects.remove(idx);
                    assert!(score_by_rects(&rects, n, m, expected) == my_score);
                }
            }
        } else if change_type == 3 {
            if rects.len() > 1 {
                let pos = rnd.gen(0..rects.len() - 1);
                rects.swap(pos, pos + 1);
                let new_score = score_by_rects(&rects, n, m, expected);
                if new_score < my_score {
                    dbg!(
                        "new best score! (by swapping rects!!!)",
                        my_score,
                        new_score
                    );
                    my_score = new_score;
                    not_changed_it = 0;
                } else {
                    rects.swap(pos, pos + 1);
                    assert!(score_by_rects(&rects, n, m, expected) == my_score);
                }
            }
        } else {
            unreachable!();
        }
    }

    dbg!(my_score);

    shrink_rects(&mut rects, n, m);
    let new_ops = gen_ops_by_solution_rects(&rects, n, m, start_whole_id);
    let final_res2 = apply_ops(&new_ops, test_case);
    let pixel_dist3 = get_pixel_distance(&final_res2.picture, &expected);
    // dbg!(final_res2.only_ops_cost + pixel_dist3);
    let r = SolutionRes {
        ops: new_ops,
        a: final_res2.picture,
        expected_score: final_res2.only_ops_cost + pixel_dist3,
    };
    dbg!("after local shift optimizations", r.expected_score);
    let after_local = optimize_colors(expected, &rects, &r.ops, start_whole_id, test_case);
    let diff = start_score - after_local.expected_score;
    dbg!(
        "after color optimizations",
        after_local.expected_score,
        start_score,
        diff
    );
    after_local
}
