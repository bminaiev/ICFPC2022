use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt::Debug,
    ops::Range,
    path::Path,
    time::Instant,
};

use algo_lib::{
    collections::{array_2d::Array2D, last_exn::LastExn},
    dbg,
    geometry::point::PointT,
    io::{
        input::Input,
        output::{output, set_global_output_to_file},
    },
    misc::{
        float_min_max::{fmax, fmin},
        rand::Random,
    },
    out, out_line,
    strings::utils::vec2str,
};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct Color([u8; 4]);

impl Color {
    pub fn dist(&self, another: &Self) -> f64 {
        let mut res = 0.0;
        for i in 0..self.0.len() {
            let delta = self.0[i] as f64 - another.0[i] as f64;
            res += delta * delta;
        }
        res.sqrt()
    }

    fn new_rand(rnd: &mut Random) -> Self {
        Self([rnd.gen(0..255u8), rnd.gen(0..255u8), rnd.gen(0..255u8), 255])
    }

    const START: Self = Self([255, 255, 255, 255]);
}

fn get_pixel_distance(my: &Array2D<Color>, expected: &Array2D<Color>) -> f64 {
    get_pixel_distance_range(my, expected, 0..my.len(), 0..my[0].len())
}

fn get_pixel_distance_range(
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
    res * 0.005
}

fn get_pixel_distance_range_one_color(
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

enum EstimateResult {
    RealResult(f64),
    Estimation(f64),
}

fn estimate_pixel_distance_range_one_color(
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

type RectId = Vec<u8>;
type Point = PointT<i32>;

fn rect_id_from_usize(x: usize) -> RectId {
    x.to_string().into_bytes()
}

fn rect_id_sub_key(id: &RectId, sub_id: usize) -> Vec<u8> {
    let mut res = id.clone();
    res.push(b'.');
    res.push(sub_id as u8 + b'0');
    res
}

#[derive(Clone)]
enum Op {
    CutPoint(RectId, Point),
    Color(RectId, Color),
    CutY(RectId, i32),
    CutX(RectId, i32),
    Merge(RectId, RectId),
}

impl Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CutPoint(arg0, arg1) => f.write_fmt(format_args!(
                "cut [{}] [{}, {}]",
                vec2str(&arg0),
                arg1.x,
                arg1.y
            )),
            Self::Color(arg0, arg1) => f.write_fmt(format_args!(
                "color [{}] [{}, {}, {}, {}]",
                vec2str(&arg0),
                arg1.0[0],
                arg1.0[1],
                arg1.0[2],
                arg1.0[3]
            )),
            Self::CutY(arg0, arg1) => {
                f.write_fmt(format_args!("cut [{}] [Y] [{}]", vec2str(&arg0), arg1))
            }
            Self::CutX(arg0, arg1) => {
                f.write_fmt(format_args!("cut [{}] [X] [{}]", vec2str(&arg0), arg1))
            }
            Self::Merge(arg0, arg1) => f.write_fmt(format_args!(
                "merge [{}] [{}]",
                vec2str(&arg0),
                vec2str(&arg1)
            )),
        }
    }
}

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

fn read_submit(path: &str) -> Vec<Op> {
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

struct OpResult {
    picture: Array2D<Color>,
    cost: f64,
}

#[derive(Clone, Copy)]
struct Rect {
    from: Point,
    to: Point,
}

impl Rect {
    pub fn new(from: Point, to: Point) -> Self {
        Self { from, to }
    }

    pub fn size(&self) -> f64 {
        let dx = (self.to.x - self.from.x) as f64;
        let dy = (self.to.y - self.from.y) as f64;
        dx * dy
    }
}

fn get_start_id() -> RectId {
    vec![b'0']
}

fn apply_ops(ops: &[Op], n: usize, m: usize) -> OpResult {
    let canvas_size = (n as f64) * (m as f64);
    let mut a = Array2D::new(Color::default(), n, m);
    for i in 0..n {
        for j in 0..m {
            for k in 0..4 {
                a[i][j].0[k] = 255;
            }
        }
    }
    let mut rects = HashMap::new();
    let start_id = get_start_id();
    rects.insert(
        start_id,
        Rect::new(Point::ZERO, Point::new(n as i32, m as i32)),
    );
    let mut cost = 0.0;

    let mut last_rect_id = 0;
    for op in ops.iter() {
        match op {
            Op::CutPoint(id, p) => {
                let r = *rects.get(id).unwrap();
                let x1 = r.from.x;
                let x2 = p.x;
                let x3 = r.to.x;
                let y1 = r.from.y;
                let y2 = p.y;
                let y3 = r.to.y;

                rects.insert(
                    rect_id_sub_key(id, 0),
                    Rect::new(Point::new(x1, y1), Point::new(x2, y2)),
                );
                rects.insert(
                    rect_id_sub_key(id, 1),
                    Rect::new(Point::new(x2, y1), Point::new(x3, y2)),
                );
                rects.insert(
                    rect_id_sub_key(id, 2),
                    Rect::new(Point::new(x2, y2), Point::new(x3, y3)),
                );
                rects.insert(
                    rect_id_sub_key(id, 3),
                    Rect::new(Point::new(x1, y2), Point::new(x2, y3)),
                );
                rects.remove(id);
                cost += (POINT_CUT_COST * canvas_size / r.size()).round();
            }
            Op::Color(id, color) => {
                let r = *rects.get(id).unwrap();
                for x in r.from.x..r.to.x {
                    for y in r.from.y..r.to.y {
                        a[x as usize][y as usize] = *color;
                    }
                }
                cost += (COLOR_COST * canvas_size / r.size()).round();
            }
            Op::CutY(id, split_y) => {
                let r = *rects.get(id).unwrap();
                rects.insert(
                    rect_id_sub_key(id, 0),
                    Rect::new(r.from, Point::new(r.to.x, *split_y)),
                );
                rects.insert(
                    rect_id_sub_key(id, 1),
                    Rect::new(Point::new(r.from.x, *split_y), r.to),
                );
                rects.remove(id);
                cost += (LINE_CUT_COST * canvas_size / r.size()).round();
            }
            Op::CutX(id, split_x) => {
                let r = *rects.get(id).unwrap();
                rects.insert(
                    rect_id_sub_key(id, 0),
                    Rect::new(r.from, Point::new(*split_x, r.to.y)),
                );
                rects.insert(
                    rect_id_sub_key(id, 1),
                    Rect::new(Point::new(*split_x, r.from.y), r.to),
                );
                rects.remove(id);
                cost += (LINE_CUT_COST * canvas_size / r.size()).round();
            }
            Op::Merge(id1, id2) => {
                let r1 = *rects.get(id1).unwrap();
                let r2 = *rects.get(id2).unwrap();
                let fr = Point::new(min(r1.from.x, r2.from.x), min(r1.from.y, r2.from.y));
                let to = Point::new(max(r1.to.x, r2.to.x), max(r1.to.y, r2.to.y));
                let new_r = Rect::new(fr, to);
                assert!(new_r.size() == r1.size() + r2.size());
                last_rect_id += 1;
                rects.insert(rect_id_from_usize(last_rect_id), new_r);
                cost += (MERGE_COST * canvas_size / fmax(r1.size(), r2.size())).round();
                rects.remove(id1);
                rects.remove(id2);
            }
        }
    }
    OpResult { picture: a, cost }
}

fn save_image(a: &Array2D<Color>, path: &str) {
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

fn save_score(score: f64, path: &str) -> bool {
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

fn save_ops(ops: &[Op], path: &str) {
    set_global_output_to_file(path);
    for op in ops.iter() {
        out_line!(format!("{:?}", op));
    }
    output().flush();
}

struct ColorPicker {
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

fn gen_coords(block_size: usize, max_value: usize) -> Vec<usize> {
    let mut res = vec![0];
    loop {
        let last = *res.last_exn();
        if last + block_size >= max_value {
            res.push(max_value);
            break;
        } else {
            res.push(last + block_size);
        }
    }
    res
}

fn p(x: usize, y: usize) -> Point {
    Point::new(x as i32, y as i32)
}

const POINT_CUT_COST: f64 = 10.0;
const LINE_CUT_COST: f64 = 7.0;
const COLOR_COST: f64 = 5.0;
const MERGE_COST: f64 = 1.0;

const FMAX: f64 = f64::MAX / 100.0;

struct ColorToCorner {
    cost: f64,
    ops: Vec<Op>,
    cur_whole_id: usize,
}

fn color_corner(n: usize, m: usize, p: Point, cur_whole_id: usize, color: Color) -> ColorToCorner {
    let main_id = rect_id_from_usize(cur_whole_id);
    let full_area = (n as f64) * (m as f64);

    let x_right = (n - p.x as usize) as f64;
    let y_up = (m - p.y as usize) as f64;
    let x_left = p.x as f64;
    let y_bottom = p.y as f64;

    let s0 = x_left * y_bottom;
    let s1 = x_right * y_bottom;
    let s2 = x_right * y_up;
    let s3 = x_left * y_up;

    if p.x == 0 || p.y == 0 {
        if p.x == 0 && p.y == 0 {
            return ColorToCorner {
                cost: COLOR_COST,
                ops: vec![Op::Color(rect_id_from_usize(cur_whole_id), color)],
                cur_whole_id,
            };
        }
        if p.x == 0 {
            let mut ops = vec![];
            ops.push(Op::CutY(main_id.clone(), p.y));
            ops.push(Op::Color(rect_id_sub_key(&main_id, 1), color));
            ops.push(Op::Merge(
                rect_id_sub_key(&main_id, 0),
                rect_id_sub_key(&main_id, 1),
            ));

            return ColorToCorner {
                cost: LINE_CUT_COST
                    + (COLOR_COST * full_area / s2).round()
                    + (MERGE_COST * full_area / fmax(s1, s2)).round(),
                ops,
                cur_whole_id: cur_whole_id + 1,
            };
        }
        assert!(p.y == 0);
        {
            let mut ops = vec![];
            ops.push(Op::CutX(main_id.clone(), p.x));
            ops.push(Op::Color(rect_id_sub_key(&main_id, 1), color));
            ops.push(Op::Merge(
                rect_id_sub_key(&main_id, 0),
                rect_id_sub_key(&main_id, 1),
            ));

            return ColorToCorner {
                cost: LINE_CUT_COST
                    + (COLOR_COST * full_area / s2).round()
                    + (MERGE_COST * full_area / fmax(s3, s2)).round(),
                ops,
                cur_whole_id: cur_whole_id + 1,
            };
        }
    }

    let mut ops = vec![];

    ops.push(Op::CutPoint(main_id.clone(), p));
    ops.push(Op::Color(rect_id_sub_key(&main_id, 2), color));
    let merge_y_first = (full_area / fmax(s0, s3) * MERGE_COST).round()
        + (full_area / fmax(s1, s2) * MERGE_COST).round()
        + (full_area / fmax(s0 + s3, s1 + s2) * MERGE_COST).round();
    let merge_x_first = (full_area / fmax(s0, s1) * MERGE_COST).round()
        + (full_area / fmax(s2, s3) * MERGE_COST).round()
        + (full_area / fmax(s0 + s1, s2 + s3) * MERGE_COST).round();
    if merge_x_first < merge_y_first {
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 0),
            rect_id_sub_key(&main_id, 1),
        ));
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 3),
            rect_id_sub_key(&main_id, 2),
        ));
    } else {
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 2),
            rect_id_sub_key(&main_id, 1),
        ));
        ops.push(Op::Merge(
            rect_id_sub_key(&main_id, 3),
            rect_id_sub_key(&main_id, 0),
        ));
    }
    ops.push(Op::Merge(
        rect_id_from_usize(cur_whole_id + 1),
        rect_id_from_usize(cur_whole_id + 2),
    ));
    let merge_back = fmin(merge_y_first, merge_x_first);
    ColorToCorner {
        cost: POINT_CUT_COST + (COLOR_COST * full_area / s2).round() + merge_back,
        ops,
        cur_whole_id: cur_whole_id + 3,
    }
}

#[derive(Clone, Copy)]
enum BestWay {
    FullColor(Color),
    SplitX(usize),
    SplitY(usize),
    SolveRec,
}

struct SmallerSolPts {
    x_map: Vec<usize>,
    y_map: Vec<usize>,
    sol: Box<Solution>,
}

struct Solution {
    dp: Array2D<Array2D<f64>>,
    dp_prev: Array2D<Array2D<BestWay>>,
    xs: Vec<usize>,
    ys: Vec<usize>,
    max_next_x_blocks: usize,
    max_next_y_blocks: usize,
    smaller_sol: Option<SmallerSolPts>,
}

impl Solution {
    pub fn iter_rects(
        &self,
        x_it: usize,
        y_it: usize,
        x_len: usize,
        y_len: usize,
        rects: &mut Vec<OneRect>,
    ) {
        let pr = self.dp_prev[x_it][y_it][x_len][y_len];
        match pr {
            BestWay::FullColor(color) => rects.push(OneRect {
                from: p(self.xs[x_it], self.ys[y_it]),
                to: p(self.xs[x_it + x_len], self.ys[y_it + y_len]),
                color,
            }),
            BestWay::SplitX(x_mid_it) => {
                self.iter_rects(x_it, y_it, x_mid_it - x_it, y_len, rects);
                self.iter_rects(x_mid_it, y_it, x_it + x_len - x_mid_it, y_len, rects);
            }
            BestWay::SplitY(y_mid_it) => {
                self.iter_rects(x_it, y_it, x_len, y_mid_it - y_it, rects);
                self.iter_rects(x_it, y_mid_it, x_len, y_it + y_len - y_mid_it, rects);
            }
            BestWay::SolveRec => {
                if let Some(smaller) = &self.smaller_sol {
                    let x_it2 = smaller.x_map[x_it + x_len];
                    let x_it = smaller.x_map[x_it];
                    let y_it2 = smaller.y_map[y_it + y_len];
                    let y_it = smaller.y_map[y_it];
                    // dbg!("go rec!", smaller.sol.max_next_x_blocks);
                    smaller
                        .sol
                        .iter_rects(x_it, y_it, x_it2 - x_it, y_it2 - y_it, rects);
                } else {
                    unreachable!();
                }
            }
        }
    }
}

fn get_mapping(xs: &[usize], smaller_xs: &[usize]) -> Vec<usize> {
    let mut res = vec![];
    for x in xs.iter() {
        res.push(smaller_xs.binary_search(x).unwrap())
    }
    res
}

fn find_solution(
    expected: &Array2D<Color>,
    xs: &[usize],
    ys: &[usize],
    color_picker: &ColorPicker,
    max_next_x_blocks: usize,
    max_next_y_blocks: usize,
    smaller_sol: Option<Box<Solution>>,
    rnd: &mut Random,
) -> Solution {
    let n = expected.len();
    let m = expected[0].len();

    let x_map = match &smaller_sol {
        None => vec![],
        Some(sol) => get_mapping(xs, &sol.xs),
    };
    let y_map = match &smaller_sol {
        None => vec![],
        Some(sol) => get_mapping(ys, &sol.ys),
    };

    let simple_elem = Array2D::new(FMAX, max_next_x_blocks + 1, max_next_y_blocks + 1);
    let simple_elem_prev = Array2D::new(
        BestWay::FullColor(Color::default()),
        max_next_x_blocks + 1,
        max_next_y_blocks + 1,
    );
    let mut dp = Array2D::new(simple_elem.clone(), xs.len() - 1, ys.len() - 1);
    let mut dp_prev = Array2D::new(simple_elem_prev.clone(), xs.len() - 1, ys.len() - 1);
    for x_it in (0..xs.len() - 1).rev() {
        for y_it in (0..ys.len() - 1).rev() {
            let x_start = xs[x_it];
            let y_start = ys[y_it];
            let mut cur_dp = simple_elem.clone();
            let mut cur_dp_prev = simple_elem_prev.clone();
            for x_len in 1..cur_dp.len() {
                for y_len in 1..cur_dp[x_len].len() {
                    let x2_it = x_it + x_len;
                    let y2_it = y_it + y_len;
                    if x2_it >= xs.len() || y2_it >= ys.len() {
                        continue;
                    }
                    let x_end = xs[x2_it];
                    let y_end = ys[y2_it];

                    if let Some(smaller_sol) = &smaller_sol {
                        // recursively
                        let x_it_smaller = x_map[x_it];
                        let y_it_smaller = y_map[y_it];
                        let x2_it_smaller = x_map[x2_it];
                        let y2_it_smaller = y_map[y2_it];
                        let x_len_smaller = x2_it_smaller - x_it_smaller;
                        let y_len_smaller = y2_it_smaller - y_it_smaller;
                        if x_len_smaller <= smaller_sol.max_next_x_blocks
                            && y_len_smaller <= smaller_sol.max_next_y_blocks
                        {
                            let cost = smaller_sol.dp[x_it_smaller][y_it_smaller][x_len_smaller]
                                [y_len_smaller];
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::SolveRec;
                            }
                        }
                    }

                    {
                        // split by x
                        for x_mid_it in x_it + 1..x2_it {
                            let cost = cur_dp[x_mid_it - x_it][y_len]
                                + dp[x_mid_it][y_it][x2_it - x_mid_it][y_len];
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::SplitX(x_mid_it);
                            }
                        }
                    }
                    {
                        // split by y
                        for y_mid_it in y_it + 1..y2_it {
                            let cost = cur_dp[x_len][y_mid_it - y_it]
                                + dp[x_it][y_mid_it][x_len][y2_it - y_mid_it];
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::SplitY(y_mid_it);
                            }
                        }
                    }

                    {
                        // color fully
                        let best_color =
                            color_picker.pick_color(p(x_start, y_start), p(x_end, y_end));
                        let cost_to_color =
                            color_corner(n, m, p(x_start, y_start), 0, best_color).cost;
                        let estimated_pixel_dist = estimate_pixel_distance_range_one_color(
                            best_color,
                            expected,
                            x_start..x_end,
                            y_start..y_end,
                            rnd,
                        );
                        const ESTIMATE_COEF: f64 = 0.8;
                        let estimated_value = match estimated_pixel_dist {
                            EstimateResult::RealResult(x) => x,
                            EstimateResult::Estimation(x) => x,
                        };
                        let coef = match estimated_pixel_dist {
                            EstimateResult::RealResult(_) => 1.0,
                            EstimateResult::Estimation(_) => ESTIMATE_COEF,
                        };
                        if estimated_value * coef + cost_to_color < cur_dp[x_len][y_len] {
                            let mut cost = cost_to_color;
                            match estimated_pixel_dist {
                                EstimateResult::RealResult(x) => cost += x,
                                EstimateResult::Estimation(_) => {
                                    cost += get_pixel_distance_range_one_color(
                                        best_color,
                                        expected,
                                        x_start..x_end,
                                        y_start..y_end,
                                    )
                                }
                            };
                            if cost < cur_dp[x_len][y_len] {
                                cur_dp[x_len][y_len] = cost;
                                cur_dp_prev[x_len][y_len] = BestWay::FullColor(best_color);
                            }
                        }
                    }
                }
            }

            dp[x_it][y_it] = cur_dp;
            dp_prev[x_it][y_it] = cur_dp_prev;
        }
    }
    let smaller_sol = match smaller_sol {
        Some(smaller_sol) => Some(SmallerSolPts {
            sol: smaller_sol,
            x_map,
            y_map,
        }),
        None => None,
    };
    Solution {
        dp,
        dp_prev,
        xs: xs.to_vec(),
        ys: ys.to_vec(),
        max_next_x_blocks,
        max_next_y_blocks,
        smaller_sol,
    }
}

fn add_more_coords(coords: &[usize], rnd: &mut Random) -> Vec<usize> {
    let mut res = vec![coords[0]];
    const BETWEEN: usize = 2;
    for i in 0..(coords.len() - 1) {
        let prev = coords[i];
        let next = coords[i + 1];
        let cnt_inside = next - prev - 1;
        if cnt_inside <= BETWEEN {
            for j in prev + 1..next {
                res.push(j);
            }
        } else {
            let mut used = vec![false; cnt_inside];
            let mut added = 0;
            while added != BETWEEN {
                let pos = rnd.gen(0..cnt_inside);
                if used[pos] {
                    continue;
                }
                used[pos] = true;
                added += 1;
            }
            for j in 0..used.len() {
                if used[j] {
                    res.push(prev + 1 + j);
                }
            }
        }
        res.push(next);
    }
    res
}

struct SolutionRes {
    a: Array2D<Color>,
    expected_score: f64,
    ops: Vec<Op>,
}

fn gen_ops_by_rects(rects: &[OneRect], field_n: usize, field_m: usize) -> Vec<Op> {
    let mut res = vec![];
    let n = rects.len();
    let mut ok_after = Array2D::new(true, n, n);
    for i in 0..n {
        for j in 0..n {
            if rects[i].from.x >= rects[j].to.x || rects[i].from.y >= rects[j].to.y {
                ok_after[i][j] = true;
            }
        }
    }
    let mut need_more_before = vec![0; n];
    for i in 0..n {
        for j in 0..n {
            if !ok_after[i][j] {
                need_more_before[j] += 1;
            }
        }
    }
    let mut queue = vec![];
    let mut used = vec![false; n];
    loop {
        let mut changed = false;
        for v in 0..n {
            if need_more_before[v] == 0 && !used[v] {
                changed = true;
                used[v] = true;
                queue.push(v);
                for j in 0..n {
                    if !ok_after[v][j] {
                        need_more_before[j] -= 1;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    assert!(queue.len() == n);
    let mut cur_whole_id = 0;
    for &rect_id in queue.iter() {
        let r = rects[rect_id];
        let to_color = color_corner(field_n, field_m, r.from, cur_whole_id, r.color);
        res.extend(to_color.ops);
        cur_whole_id = to_color.cur_whole_id;
    }

    res
}

fn solve_one(expected: &Array2D<Color>, block_size: usize) -> SolutionRes {
    let color_picker = ColorPicker::new(&expected);
    let n = expected.len();
    let m = expected[0].len();
    // const BLOCK_SIZE: usize = 6;
    let xs = gen_coords(block_size, n);
    let ys = gen_coords(block_size, m);
    let blocks_n = xs.len() - 1;
    let blocks_m = ys.len() - 1;

    let mut rnd = Random::new(7877881);
    let more_xs = add_more_coords(&xs, &mut rnd);
    let more_ys = add_more_coords(&ys, &mut rnd);

    let more_xs_2 = add_more_coords(&more_xs, &mut rnd);
    let more_ys_2 = add_more_coords(&more_ys, &mut rnd);
    // dbg!(&xs);
    // dbg!(&more_xs);
    // dbg!(&more_xs_2);

    const MAX_NEXT_BLOCKS: usize = 15;

    let smallest_solution = find_solution(
        expected,
        &more_xs_2,
        &more_ys_2,
        &color_picker,
        MAX_NEXT_BLOCKS,
        MAX_NEXT_BLOCKS,
        None,
        &mut rnd,
    );

    let small_solution = find_solution(
        expected,
        &more_xs,
        &more_ys,
        &color_picker,
        MAX_NEXT_BLOCKS,
        MAX_NEXT_BLOCKS,
        Some(Box::new(smallest_solution)),
        // None,
        &mut rnd,
    );
    // dbg!("small solution done!");
    let solution = find_solution(
        expected,
        &xs,
        &ys,
        &color_picker,
        blocks_n,
        blocks_m,
        Some(Box::new(small_solution)),
        &mut rnd,
    );

    let mut rects = vec![];
    solution.iter_rects(0, 0, xs.len() - 1, ys.len() - 1, &mut rects);

    dbg!(rects.len());

    let ops = gen_ops_by_rects(&rects, n, m);
    // for r in rects.iter() {
    //     dbg!(
    //         (r.to.x - r.from.x) as usize / BLOCK_SIZE,
    //         (r.to.y - r.from.y) as usize / BLOCK_SIZE
    //     );
    // }

    let mut my_picture = Array2D::new(Color::START, n, m);
    for r in rects.iter() {
        let c = r.color; //Color::new_rand(&mut rnd);
        for x in r.from.x..r.to.x {
            for y in r.from.y..r.to.y {
                my_picture[x as usize][y as usize] = c;
            }
        }
    }

    let ops_res = apply_ops(&ops, n, m);
    // dbg!(ops_res.cost + get_pixel_distance(&ops_res.picture, expected));
    for i in 0..n {
        for j in 0..m {
            assert_eq!(my_picture[i][j], ops_res.picture[i][j]);
        }
    }

    let res = solution.dp[0][0][xs.len() - 1][ys.len() - 1];
    dbg!(res);
    SolutionRes {
        a: my_picture,
        expected_score: res,
        ops,
    }
}

#[derive(Clone, Copy)]
struct OneRect {
    from: Point,
    to: Point,
    color: Color,
}

fn read_case(test_id: usize) -> Array2D<Color> {
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

fn solve_case(test_id: usize, block_size: usize) {
    let expected = read_case(test_id);
    let solution = solve_one(&expected, block_size);

    if save_score(
        solution.expected_score,
        &format!("../scores/{}.txt", test_id),
    ) {
        save_image(&solution.a, &format!("../images/{}.res.png", test_id));
        save_ops(&solution.ops, &format!("../outputs/{}.isl", test_id))
    }
}

fn show_case(test_id: usize) {
    let expected = read_case(test_id);
    let n = expected.len();
    let m = expected[0].len();
    let submit = read_submit(&format!("../outputs/{}.isl", test_id));
    let op_res = apply_ops(&submit, n, m);
    let dist = get_pixel_distance(&op_res.picture, &expected);
    dbg!(dist + op_res.cost);
    save_image(&op_res.picture, &format!("../images/{}.res.png", test_id))
}

fn solve_all() {
    for &block_size in [50, 20, 15, 10, 8, 7, 6, 5].iter() {
        for task_id in 1..=25 {
            dbg!(task_id, block_size);
            let start = Instant::now();
            solve_case(task_id, block_size);
            dbg!(start.elapsed());
        }
    }
}

fn main() {
    solve_all();
    // let start = Instant::now();
    // solve_case(4);
    // dbg!(start.elapsed());
}
