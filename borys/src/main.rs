use std::time::Instant;

use algo_lib::{dbg, geometry::point::PointT, misc::rand::Random};

mod color;

use crate::{
    interpreter::apply_ops,
    local_optimizations::{optimize_colors, optimize_positions},
    ops_by_rects::gen_rects_by_ops,
    pixel_dist::get_pixel_distance,
    readings::{read_case, read_submit},
    savings::{save_image, save_ops, save_score, save_solution},
    solver::{solve_one, SolutionRes},
};
mod color_corner;
mod color_picker;
mod consts;
mod interpreter;
mod local_optimizations;
mod op;
mod ops_by_rects;
mod pixel_dist;
mod readings;
mod rect_id;
mod savings;
mod solver;
mod utils;

type Point = PointT<i32>;

fn solve_case(test_id: usize, block_size: usize, use_third_layer: bool) {
    let expected = read_case(test_id);
    let solution = solve_one(&expected, block_size, use_third_layer);
    save_solution(test_id, &solution);
}

fn show_case(test_id: usize) {
    let expected = read_case(test_id);
    let n = expected.len();
    let m = expected[0].len();
    let submit = read_submit(&format!("../outputs/{}.isl", test_id));
    let op_res = apply_ops(&submit, n, m);
    let dist = get_pixel_distance(&op_res.picture, &expected);
    dbg!(dist + op_res.only_ops_cost);
    save_image(&op_res.picture, &format!("../images/{}.res.png", test_id))
}

fn solve_all() {
    for &block_size in [50, 20, 15, 10, 8, 7, 6, 5].iter() {
        for task_id in 1..=25 {
            dbg!(task_id, block_size);
            let start = Instant::now();
            solve_case(task_id, block_size, true);
            dbg!(start.elapsed());
        }
    }
}

fn solve_fast(task_id: usize) {
    let start = Instant::now();
    solve_case(task_id, 20, false);
    dbg!(start.elapsed());
}

fn local_optimize(test_id: usize) {
    let mut rnd = Random::new_time_seed();
    let start = Instant::now();
    let expected = read_case(test_id);
    let ops = read_submit(&format!("../outputs/{}.isl", test_id));
    let rects = gen_rects_by_ops(&ops, expected.len(), expected[0].len());
    let new_sol = optimize_positions(&expected, &rects, &ops, &mut rnd);
    save_solution(test_id, &new_sol);
    dbg!(start.elapsed());
}

fn main() {
    // solve_all();
    const TEST_ID: usize = 25;
    loop {
        dbg!("NEXT ITERATION!!!");
        for test_id in 1..=25 {
            dbg!(test_id);
            local_optimize(test_id);
        }
    }
    // let start = Instant::now();
    // solve_case(4);
    // dbg!(start.elapsed());
}
