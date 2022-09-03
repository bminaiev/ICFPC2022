use algo_lib::{collections::array_2d::Array2D, io::input::Input, strings::utils::vec2str};

use crate::{color::Color, op::Op, rect_id::RectId, Point};

fn remove_prefix(s: Vec<u8>, c: char) -> Vec<u8> {
    assert!(s[0] == c as u8);
    s[1..].to_vec()
}

fn remove_suffix(s: Vec<u8>, c: char) -> Vec<u8> {
    let n = s.len() - 1;
    assert!(s[n] == c as u8);
    s[..n].to_vec()
}

fn read_id(input: &mut Input) -> RectId {
    let id = input.string();
    assert!(id[0] == b'[');
    assert!(id[id.len() - 1] == b']');
    let id = remove_suffix(remove_prefix(id, '['), ']');
    id
}

fn parse_mid_int(s: &[u8]) -> i32 {
    let mut start = 0;
    while !(s[start] >= b'0' && s[start] <= b'9') {
        start += 1;
    }
    let mut end = s.len() - 1;
    while !(s[end] >= b'0' && s[end] <= b'9') {
        end -= 1;
    }
    vec2str(&s[start..=end]).parse().unwrap()
}

fn read_next_int(input: &mut Input) -> i32 {
    let s = input.string();
    parse_mid_int(&s)
}

fn read_color(input: &mut Input) -> Color {
    let r = read_next_int(input) as u8;
    let g = read_next_int(input) as u8;
    let b = read_next_int(input) as u8;
    let a = read_next_int(input) as u8;
    Color([r, g, b, a])
}

pub fn read_submit(path: &str) -> Vec<Op> {
    let mut res = vec![];
    let mut input = Input::new_file(path);
    while input.has_more_elements() {
        let cmd = input.string_as_string();
        if cmd == "cut" {
            let id = read_id(&mut input);
            let token = input.string();
            if token[1] == b'Y' || token[1] == b'X' {
                if token[1] == b'Y' {
                    let y = read_next_int(&mut input);
                    res.push(Op::CutY(id, y))
                } else {
                    assert!(token[1] == b'X');
                    let x = read_next_int(&mut input);
                    res.push(Op::CutX(id, x))
                }
            } else {
                let x = parse_mid_int(&token);
                let y = read_next_int(&mut input);

                res.push(Op::CutPoint(id, Point::new(x, y)));
            }
        } else if cmd == "color" {
            let id = read_id(&mut input);
            let color = read_color(&mut input);
            res.push(Op::Color(id, color));
        } else {
            assert!(false);
        }
    }
    res
}

pub fn read_case(test_id: usize) -> Array2D<Color> {
    let mut input = Input::new_file(format!("../inputs/{}.txt", test_id));
    let m = input.usize();
    let n = input.usize();
    let mut expected = Array2D::new(Color::default(), n, m);
    for i in 0..m {
        for j in 0..n {
            for k in 0..4 {
                expected[j][m - i - 1].0[k] = input.u32() as u8;
            }
        }
    }
    expected
}
