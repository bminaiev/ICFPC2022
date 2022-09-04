use algo_lib::dbg;

use crate::{
    consts::MERGE_COST,
    interpreter::apply_ops,
    op::Op,
    rect_id::{rect_id_from_usize, rect_id_sub_key, rect_id_to_usize, RectId},
    test_case::{self, TestCase},
};

pub struct MergeResult {
    pub ops: Vec<Op>,
    pub last_block_id: usize,
    pub expected_score: f64,
}

fn merge_line(names: &[RectId], mut whole_id: usize) -> MergeResult {
    let mut cur_name = names[0].clone();
    let mut ops = vec![];
    for reg in names[1..].iter() {
        ops.push(Op::Merge(cur_name.clone(), reg.clone()));
        whole_id += 1;
        cur_name = rect_id_from_usize(whole_id);
    }
    MergeResult {
        ops,
        last_block_id: rect_id_to_usize(&cur_name),
        expected_score: 0.0,
    }
}

fn merge_cost(rect_size: usize, n: usize) -> f64 {
    let n = n as f64;
    let rect_size = rect_size as f64;
    let canvas_size = n * n;
    (MERGE_COST * canvas_size / rect_size).round()
}

fn line_cut_cost(rect_size: usize, n: usize, test_case: &TestCase) -> f64 {
    let n = n as f64;
    let rect_size = rect_size as f64;
    let canvas_size = n * n;
    (test_case.get_line_cut_cost() * canvas_size / rect_size).round()
}

fn merge_line_cost(block_size: usize, n: usize) -> f64 {
    let tot_blocks = n / block_size;
    let mut cur_sz = block_size * block_size;
    let mut res = 0.0;
    for _cnt in 2..=tot_blocks {
        res += merge_cost(cur_sz, n);
        cur_sz += block_size * block_size;
    }
    res
}

fn calc_best_merge_cost_fixed_first_lines(
    block_size: usize,
    n: usize,
    first_lines: usize,
    test_case: &TestCase,
) -> f64 {
    let line_cost = merge_line_cost(block_size, n);
    let tot_blocks = n / block_size;
    let mut res = line_cost * (first_lines as f64);
    for lines in 1..first_lines {
        res += merge_cost(lines * n * block_size, n);
    }
    let mut cur_rect_size = first_lines * block_size * n;
    for _col in 1..tot_blocks {
        res += line_cut_cost(cur_rect_size, n, test_case);
        cur_rect_size -= first_lines * block_size * block_size;
    }
    assert_eq!(cur_rect_size, first_lines * block_size * block_size);
    let mut merge_one_col = 0.0;
    for row in first_lines..tot_blocks {
        merge_one_col += merge_cost(row * block_size * block_size, n);
    }
    res += merge_one_col * (tot_blocks as f64);
    for col in 1..tot_blocks {
        res += merge_cost(col * block_size * n, n);
    }
    res
}

fn calc_best_merge_cost(block_size: usize, n: usize, test_case: &TestCase) -> f64 {
    let mut res = std::f64::MAX;
    let tot_blocks = n / block_size;
    for first_lines in 1..=tot_blocks {
        let cur_cost =
            calc_best_merge_cost_fixed_first_lines(block_size, n, first_lines, test_case);
        dbg!(first_lines, cur_cost);
        if cur_cost < res {
            res = cur_cost;
        }
    }
    res
}

fn gen_ops(test_case: &TestCase, first_lines: usize) -> MergeResult {
    let n_regions = test_case.regions.len();
    let mut last_block_id = n_regions - 1;
    let mut regions = test_case.regions.clone();
    regions.sort_by_key(|r| r.rect.from);

    let n = test_case.get_size().0;
    let block_size = (regions[0].rect.to.x - regions[0].rect.from.x) as usize;
    let tot_blocks = n / block_size;

    let mut ops = vec![];
    let mut col_ids = vec![];
    {
        // merge each of the [first_lines]
        for line in 0..first_lines {
            let line_regions = &regions[line * tot_blocks..(line + 1) * tot_blocks];
            let names: Vec<_> = line_regions.iter().map(|r| r.name.clone()).collect();
            let res = merge_line(&names, last_block_id);
            ops.extend(res.ops);
            col_ids.push(res.last_block_id);
            last_block_id = res.last_block_id;
        }
    }
    {
        // merge [first_lines] into one block
        let mut cur_id = col_ids[0];
        for &blk in col_ids[1..].iter() {
            ops.push(Op::Merge(
                rect_id_from_usize(cur_id),
                rect_id_from_usize(blk),
            ));
            last_block_id += 1;
            cur_id = last_block_id;
        }
    }
    let mut row_ids = vec![];
    {
        let mut cur_merged_block_id = rect_id_from_usize(last_block_id);
        // split merged block into rows
        for line in 1..tot_blocks {
            ops.push(Op::CutY(
                cur_merged_block_id.clone(),
                (line * block_size) as i32,
            ));
            row_ids.push(rect_id_sub_key(&cur_merged_block_id, 0));
            cur_merged_block_id = rect_id_sub_key(&cur_merged_block_id, 1);
        }
        row_ids.push(cur_merged_block_id);
    }
    assert_eq!(row_ids.len(), tot_blocks);
    {
        // merge each small block into row
        for reg in regions[first_lines * tot_blocks..].iter() {
            let row_id = reg.rect.from.y as usize / block_size;
            ops.push(Op::Merge(row_ids[row_id].clone(), reg.name.clone()));
            last_block_id += 1;
            row_ids[row_id] = rect_id_from_usize(last_block_id);
        }
    }
    {
        // merge rows
        let mut merged_block_id = row_ids[0].clone();
        for blk in row_ids[1..].iter() {
            ops.push(Op::Merge(merged_block_id, blk.clone()));
            last_block_id += 1;
            merged_block_id = rect_id_from_usize(last_block_id);
        }
    }
    let after_apply = apply_ops(&ops, test_case);

    MergeResult {
        ops: ops,
        last_block_id,
        expected_score: after_apply.only_ops_cost,
    }
}

pub fn merge(test_case: &TestCase) -> MergeResult {
    let n_regions = test_case.regions.len();
    if n_regions == 1 {
        return MergeResult {
            ops: vec![],
            last_block_id: 0,
            expected_score: 0.0,
        };
    }

    let regions = test_case.regions.clone();
    let block_size = (regions[0].rect.to.x - regions[0].rect.from.x) as usize;
    let n = test_case.get_size().0;
    let tot_blocks = n / block_size;

    let mut best_res = gen_ops(test_case, 1);
    for first_lines in 2..=tot_blocks {
        let cur_res = gen_ops(test_case, first_lines);
        if cur_res.expected_score < best_res.expected_score {
            best_res = cur_res;
        }
    }

    dbg!("expected merge cost", best_res.expected_score);

    best_res
}
