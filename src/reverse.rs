#[inline]
pub fn reverse(array: &mut Vec<Vec<f64>>, n: usize) {
    let sub = array.len().saturating_sub(n);
    array[sub..].reverse()
}
