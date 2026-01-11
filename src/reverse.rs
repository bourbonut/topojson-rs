#[inline]
pub fn reverse(array: &mut Vec<[f64; 2]>, n: usize) {
    let sub = array.len().saturating_sub(n);
    array[sub..].reverse()
}
