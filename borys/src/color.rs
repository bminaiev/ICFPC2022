use algo_lib::misc::rand::Random;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub fn dist(&self, another: &Self) -> f64 {
        let mut res = 0.0;
        for i in 0..self.0.len() {
            let delta = self.0[i] as f64 - another.0[i] as f64;
            res += delta * delta;
        }
        res.sqrt()
    }

    pub fn new_rand(rnd: &mut Random) -> Self {
        Self([rnd.gen(0..255u8), rnd.gen(0..255u8), rnd.gen(0..255u8), 255])
    }

    pub const START: Self = Self([255, 255, 255, 255]);
}
