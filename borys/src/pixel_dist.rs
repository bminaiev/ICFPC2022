use std::ops::Range;

use algo_lib::{collections::array_2d::Array2D, misc::rand::Random};

use crate::color::Color;

pub enum EstimateResult {
    RealResult(f64),
    Estimation(f64),
}

pub fn estimate_pixel_distance_range_one_color(
    color: Color,
    expected: &Array2D<Color>,
    xs: Range<usize>,
    ys: Range<usize>,
    rnd: &mut Random,
) -> EstimateResult {
    let area = (xs.end - xs.start) * (ys.end - ys.start);
    if area < 50 {
        return EstimateResult::RealResult(get_pixel_distance_range_one_color(
            color, expected, xs, ys,
        ));
    }
    let mut res = 0.0;
    const MAX_TRIES: usize = 20;
    for _ in 0..MAX_TRIES {
        let x = rnd.gen(xs.clone());
        let y = rnd.gen(ys.clone());
        res += color.dist(&expected[x][y]);
    }
    EstimateResult::Estimation(res * (area as f64) / (MAX_TRIES as f64) * 0.005)
}

pub fn get_pixel_distance(my: &Array2D<Color>, expected: &Array2D<Color>) -> f64 {
    get_pixel_distance_range(my, expected, 0..my.len(), 0..my[0].len())
}

pub const PIXEL_DIST_COEF: f64 = 0.005;

pub fn get_pixel_distance_range(
    my: &Array2D<Color>,
    expected: &Array2D<Color>,
    xs: Range<usize>,
    ys: Range<usize>,
) -> f64 {
    let mut res = 0.0;
    for i in xs {
        for j in ys.clone() {
            res += my[i][j].dist(&expected[i][j]);
        }
    }
    res * PIXEL_DIST_COEF
}

pub fn get_pixel_distance_range_one_color(
    color: Color,
    expected: &Array2D<Color>,
    xs: Range<usize>,
    ys: Range<usize>,
) -> f64 {
    let mut res = 0.0;
    for i in xs {
        for j in ys.clone() {
            res += color.dist(&expected[i][j]);
        }
    }
    res * 0.005
}
