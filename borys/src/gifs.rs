use crate::{
    interpreter::{apply_ops, Frame},
    readings::{read_case, read_submit},
    Point,
};

const W: usize = 10;
const N: usize = 20;
const WIDTH: usize = 401;

fn gen_picture_by_frame(frame: &Frame) -> Vec<u8> {
    let mut res = vec![0; WIDTH * WIDTH];

    let id = |x: usize, y: usize| -> usize {
        assert!(x < WIDTH);
        assert!(y < WIDTH);
        x * WIDTH + y
    };
    let conv_point = |p: Point| -> (usize, usize) { (p.x as usize, p.y as usize) };

    for rect in frame.rects.iter() {
        let (x1, y1) = conv_point(rect.from);
        let (x2, y2) = conv_point(rect.to);

        for x in x1..=x2 {
            res[id(x, y1)] = 1;
            res[id(x, y2)] = 1;
        }

        for y in y1..=y2 {
            res[id(x1, y)] = 1;
            res[id(x2, y)] = 1;
        }
    }

    res
}

pub fn gen_gif(test_id: usize) {
    let test_case = read_case(test_id);
    let ops = read_submit(&format!("../outputs/{}.isl", test_id));
    let res = apply_ops(&ops, &test_case, true);

    use gif::{Encoder, Frame, Repeat};
    use std::borrow::Cow;
    use std::fs::File;

    let color_map = &[0xFF, 0xFF, 0xFF, 0, 0, 0];

    let w16 = WIDTH as u16;

    let mut image = File::create("test.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, w16, w16, color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();
    for my_frame in res.frames.iter() {
        let state = gen_picture_by_frame(my_frame);
        let mut frame = Frame::default();
        frame.width = w16;
        frame.height = w16;
        frame.buffer = Cow::Borrowed(&*state);
        encoder.write_frame(&frame).unwrap();
        if my_frame.rects.len() == 1 {
            // only show merging steps
            break;
        }
    }
}
