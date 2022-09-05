use std::time::Instant;

use algo_lib::{dbg, geometry::point::PointT, misc::rand::Random};

mod color;

use crate::{
    analyze::analyze,
    interpreter::apply_ops,
    local_optimizations::{optimize_colors, optimize_positions},
    ops_by_rects::gen_rects_by_ops,
    pixel_dist::get_pixel_distance,
    readings::{read_case, read_submit},
    really_local_optimizations::really_optimize_positions,
    rotator::Rotator,
    savings::{save_image, save_ops, save_score, save_solution},
    solver::{solve_one, SolutionRes},
    task40::task40,
};
mod analyze;
mod color_corner;
mod color_picker;
mod consts;
mod interpreter;
mod local_optimizations;
mod merger;
mod op;
mod ops_by_rects;
mod pixel_dist;
mod readings;
mod really_local_optimizations;
mod rect_id;
mod rotator;
mod savings;
mod solver;
mod task40;
mod test_case;
mod utils;

type Point = PointT<i32>;

fn solve_case(test_id: usize, block_size: usize, use_third_layer: bool, rotations: i32) {
    let rotator = {
        let test_case = read_case(test_id);
        Rotator::new_fixed_rotations(&test_case, rotations)
    };
    let solution = solve_one(&rotator.test_case, block_size, use_third_layer);
    save_solution(test_id, &rotator.rotate_sol(solution));
}

// fn show_case(test_id: usize) {
//     let expected = read_case(test_id).expected;
//     let n = expected.len();
//     let m = expected[0].len();
//     let submit = read_submit(&format!("../outputs/{}.isl", test_id));
//     let op_res = apply_ops(&submit, n, m);
//     let dist = get_pixel_distance(&op_res.picture, &expected);
//     dbg!(dist + op_res.only_ops_cost);
//     save_image(&op_res.picture, &format!("../images/{}.res.png", test_id))
// }

fn solve_all() {
    for &block_size in [50, 20, 15, 10, 8, 7, 6, 5].iter() {
        for task_id in 1..=25 {
            dbg!(task_id, block_size);
            let start = Instant::now();
            solve_case(task_id, block_size, true, 0);
            dbg!(start.elapsed());
        }
    }
}

fn solve_fast(task_id: usize) {
    let start = Instant::now();
    solve_case(task_id, 20, false, 3);
    dbg!(start.elapsed());
}

fn local_optimize(test_id: usize) {
    let mut rnd = Random::new_time_seed();
    let start = Instant::now();

    let rotator = {
        let test_case = read_case(test_id);
        let ops = read_submit(&format!("../outputs/{}.isl", test_id));
        {
            // maybe we downloaded something better than we have locally? Update score
            let sol = SolutionRes::new_from_ops(&test_case, &ops);
            save_solution(test_id, &sol);
        }
        Rotator::new(&test_case, &ops)
    };
    let expected = &rotator.test_case.expected;
    let rects = gen_rects_by_ops(&rotator.ops, expected.len(), expected[0].len());
    let new_sol_before_rotation = optimize_positions(&rects, &mut rnd, &rotator.test_case);
    let new_sol = rotator.rotate_sol(new_sol_before_rotation);
    save_solution(test_id, &new_sol);
    dbg!(start.elapsed());
}

fn really_local_optimize(test_id: usize) {
    let mut rnd = Random::new_time_seed();
    let start = Instant::now();

    let start_test_case = read_case(test_id);
    let rotator = {
        let ops = read_submit(&format!("../outputs/{}.isl", test_id));
        {
            // maybe we downloaded something better than we have locally? Update score
            let sol = SolutionRes::new_from_ops(&start_test_case, &ops);
            save_solution(test_id, &sol);
        }
        Rotator::new(&start_test_case, &ops)
    };
    let expected = &rotator.test_case.expected;
    let rects = gen_rects_by_ops(&rotator.ops, expected.len(), expected[0].len());
    let new_sol_before_rotation = really_optimize_positions(
        &rects,
        &mut rnd,
        &rotator.test_case,
        &rotator,
        &start_test_case,
    );
    let new_sol = rotator.rotate_sol(new_sol_before_rotation);
    save_solution(test_id, &new_sol);
    dbg!(start.elapsed());
}

fn analyze_here(test_id: usize) {
    let mut rnd = Random::new_time_seed();
    let start = Instant::now();

    let start_test_case = read_case(test_id);
    let rotator = {
        let ops = read_submit(&format!("../outputs/{}.isl", test_id));
        {
            // maybe we downloaded something better than we have locally? Update score
            let sol = SolutionRes::new_from_ops(&start_test_case, &ops);
            save_solution(test_id, &sol);
        }
        Rotator::new(&start_test_case, &ops)
    };
    let expected = &rotator.test_case.expected;
    let rects = gen_rects_by_ops(&rotator.ops, expected.len(), expected[0].len());
    let new_sol_before_rotation = analyze(
        &rects,
        &mut rnd,
        &rotator.test_case,
        &rotator,
        &start_test_case,
    );
    let new_sol = rotator.rotate_sol(new_sol_before_rotation);
    save_solution(test_id, &new_sol);
    dbg!(start.elapsed());
}

fn task40_solver() {
    let test_id = 40;
    let mut rnd = Random::new_time_seed();
    let start = Instant::now();

    let start_test_case = read_case(test_id);
    let rotator = {
        let ops = read_submit(&format!("../outputs/{}.isl", 205)); // good constant!
        Rotator::new(&start_test_case, &ops)
    };
    let expected = &rotator.test_case.expected;
    let rects = gen_rects_by_ops(&rotator.ops, expected.len(), expected[0].len());
    let new_sol_before_rotation = task40(
        &rects,
        &mut rnd,
        &rotator.test_case,
        &rotator,
        &start_test_case,
    );
    // let new_sol = rotator.rotate_sol(new_sol_before_rotation);
    // save_solution(test_id, &new_sol);
    // dbg!(start.elapsed());
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    dbg!(args);
    if args.len() == 3 && args[1] == "swaps" {
        let test_id: usize = args[2].parse().unwrap();
        dbg!("run specific swaps...", test_id);
        really_local_optimize(test_id);
        return;
    }

    {
        task40_solver();
        if true {
            return;
        }
    }

    // solve_all();
    // const TEST_ID: usize = 40;
    // analyze_here(TEST_ID);
    // if true {
    //     return;
    // }
    // solve_fast(TEST_ID);
    // solve_case(TEST_ID, 4, true, 2);
    // loop {
    //     local_optimize(TEST_ID);
    // }
    // loop {
    //     dbg!("NEXT ITERATION!!!");
    loop {
        // let bad_tests = [4, 5, 40, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35];
        for &test_id in [9, 38, 39].iter() {
            // if bad_tests.contains(&test_id) {
            //     continue;
            // }
            dbg!(test_id);
            // loop {
            local_optimize(test_id);
            // }
            // solve_fast(test_id);
            // solve_case(test_id, 6, true);
        }
    }
    // }
    // let start = Instant::now();
    // solve_case(4);
    // dbg!(start.elapsed());
}
