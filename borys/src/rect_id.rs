use algo_lib::strings::utils::vec2str;

pub type RectId = Vec<u8>;

pub fn rect_id_from_usize(x: usize) -> RectId {
    x.to_string().into_bytes()
}

pub fn rect_id_to_usize(s: &RectId) -> usize {
    vec2str(s).parse().unwrap()
}

pub fn rect_id_sub_key(id: &RectId, sub_id: usize) -> Vec<u8> {
    let mut res = id.clone();
    res.push(b'.');
    res.push(sub_id as u8 + b'0');
    res
}

pub fn rect_id_get_start() -> RectId {
    vec![b'0']
}

pub fn rect_id_to_vec(id: &RectId) -> Vec<usize> {
    vec2str(id).split(".").map(|x| x.parse().unwrap()).collect()
}

pub fn rect_id_from_vec(a: &[usize]) -> RectId {
    a.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(".")
        .into_bytes()
}
