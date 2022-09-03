use algo_lib::{
    misc::{group_by::GroupByTrait, min_max::UpdateMinMax},
    strings::utils::vec2str,
};

use crate::{
    op::Op,
    rect_id::{rect_id_from_usize, rect_id_to_usize, RectId},
    test_case::TestCase,
};

pub struct MergeResult {
    pub ops: Vec<Op>,
    pub last_block_id: usize,
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
    }
}

pub fn merge(test_case: &TestCase) -> MergeResult {
    let n_regions = test_case.regions.len();
    let mut ops = vec![];
    let mut whole_id = n_regions - 1;
    let mut regions = test_case.regions.clone();
    regions.sort_by_key(|r| r.rect.from);
    let mut line_ids = vec![];
    for one_line in regions.group_by_(|r1, r2| r1.rect.from.x == r2.rect.from.x) {
        let names: Vec<_> = one_line.iter().map(|r| r.name.clone()).collect();
        let res = merge_line(&names, whole_id);
        ops.extend(res.ops);
        whole_id.update_max(res.last_block_id);
        line_ids.push(rect_id_from_usize(res.last_block_id.clone()));
    }
    let res = merge_line(&line_ids, whole_id);
    ops.extend(res.ops);

    MergeResult {
        ops,
        last_block_id: res.last_block_id,
    }
}
