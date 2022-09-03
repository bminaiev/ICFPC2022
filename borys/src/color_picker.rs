use algo_lib::collections::array_2d::Array2D;

use crate::{color::Color, Point};

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
