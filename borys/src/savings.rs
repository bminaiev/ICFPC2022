use std::path::Path;

use algo_lib::{
    collections::array_2d::Array2D,
    io::{
        input::Input,
        output::{output, set_global_output_to_file},
    },
    out, out_line,
};

use crate::{color::Color, op::Op};

pub fn save_image(a: &Array2D<Color>, path: &str) {
    let width = a[0].len();
    let height = a.len();

    let mut buffer = vec![];
    for x in 0..height {
        for y in 0..width {
            let c = a[y][height - 1 - x];
            for k in c.0.iter() {
                buffer.push(*k);
            }
        }
    }

    image::save_buffer(
        Path::new(path),
        &buffer,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap()
}

pub fn save_score(score: f64, path: &str) -> bool {
    if Path::new(path).exists() {
        let mut input = Input::new_file(path);
        let prev_score = input.f64().0;
        if prev_score < score {
            dbg!("will not update score", prev_score, score);
            return false;
        }
    }

    set_global_output_to_file(path);
    out_line!(score);
    output().flush();
    true
}

pub fn save_ops(ops: &[Op], path: &str) {
    set_global_output_to_file(path);
    for op in ops.iter() {
        out_line!(format!("{:?}", op));
    }
    output().flush();
}
