use algo_lib::collections::array_2d::Array2D;

use crate::{color::Color, pixel_dist::PIXEL_DIST_COEF, Point};

pub struct ColorPicker {
    pref_sums: Vec<Array2D<f64>>,
    n: usize,
    m: usize,
}

impl ColorPicker {
    pub fn new(expected: &Array2D<Color>) -> Self {
        let mut pref_sums = vec![];
        let n = expected.len();
        let m = expected[0].len();
        for it in 0..4 {
            let mut pref_sum = Array2D::new(0.0, n + 1, m + 1);
            for i in 0..n {
                for j in 0..m {
                    pref_sum[i + 1][j + 1] += pref_sum[i][j + 1];
                    pref_sum[i + 1][j + 1] += pref_sum[i + 1][j];
                    pref_sum[i + 1][j + 1] -= pref_sum[i][j];
                    pref_sum[i + 1][j + 1] += expected[i][j].0[it] as f64;
                }
            }
            pref_sums.push(pref_sum);
        }
        Self {
            pref_sums,
            n: expected.len(),
            m: expected[0].len(),
        }
    }

    pub fn pick_color(&self, fr: Point, to: Point) -> Color {
        assert!(to.x <= self.n as i32);
        assert!(to.y <= self.m as i32);
        let mut res = Color::default();
        let dx = (to.x - fr.x) as f64;
        let dy = (to.y - fr.y) as f64;
        let area = dx * dy;
        for it in 0..4 {
            let mut sum = self.pref_sums[it][to.x as usize][to.y as usize];
            sum -= self.pref_sums[it][to.x as usize][fr.y as usize];
            sum -= self.pref_sums[it][fr.x as usize][to.y as usize];
            sum += self.pref_sums[it][fr.x as usize][fr.y as usize];
            let av_color = (sum / area).round();
            assert!(av_color >= 0.0);
            assert!(av_color <= 255.0);
            res.0[it] = av_color as u8;
        }
        res
    }
}

fn color_dist(colors: &[Color], my: Color) -> f64 {
    let mut res = 0.0;
    for c in colors.iter() {
        res += c.dist(&my);
    }
    res * PIXEL_DIST_COEF
}

pub fn find_best_color(colors: &[Color], mut my: Color) -> Color {
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

pub fn estimate_best_color(colors: &[Color]) -> Color {
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
