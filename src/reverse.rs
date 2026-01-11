#[inline]
pub fn reverse(array: &mut [[f64; 2]], n: usize) {
    let sub = array.len().saturating_sub(n);
    array[sub..].reverse()
}
