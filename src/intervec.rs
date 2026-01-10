use pyo3::{prelude::*, types::PyList};
use std::array::from_fn;

#[derive(Clone, PartialEq, Debug)]
pub struct InterVec<T, const N: usize> {
    indices: Vec<[usize; N]>,
    data: Vec<T>,
    length: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RefInterVec<'a, T, const N: usize> {
    indices: &'a [[usize; N]],
    data: &'a [T],
    length: usize,
    depth: usize,
}

pub struct IterData<'a, T, const N: usize> {
    indices: &'a [[usize; N]],
    data: &'a [T],
    length: usize,
    index: usize,
    depth: usize,
}

pub trait InterIterator<'a, T, const N: usize> {
    fn iter(&self) -> IterRef<'_, T, N>;
    fn iter_data(&self) -> IterData<'_, T, N>;
    fn iter_flatten(&self) -> IterFlatten<'_, T, N>;
}

impl<'a, T, const N: usize> Iterator for IterData<'a, T, N> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }
        let (depth_indices, right_indices) = self.indices[self.index].split_at(self.depth);
        let depth_index = right_indices.first().unwrap();
        let start = self.index;
        self.index += 1;

        while self.index < self.length {
            let (next_indices, next_right) = self.indices[self.index].split_at(self.depth);
            let next_index = next_right.first().unwrap();
            if next_index == depth_index && next_indices == depth_indices {
                self.index += 1;
            } else {
                break;
            }
        }
        let end = self.index;
        Some(&self.data[start..end])
    }
}

impl<'a, T, const N: usize> RefInterVec<'a, T, N> {
    pub fn get(&self, index: usize) -> Option<RefInterVec<'_, T, N>> {
        if index >= self.length {
            return None;
        }

        let mut start = 0;
        while start < self.indices.len() && self.indices[start][self.depth] != index {
            start += 1;
        }
        let mut end = start;
        while end < self.indices.len() && self.indices[end][self.depth] == index {
            end += 1;
        }

        let length = if self.depth + 1 <= N {
            self.indices[end - 1][self.depth + 1] + 1
        } else {
            1
        };
        Some(RefInterVec {
            indices: &self.indices[start..end],
            data: &self.data[start..end],
            length,
            depth: 1,
        })
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

impl<'a, T, const N: usize> InterIterator<'a, T, N> for RefInterVec<'a, T, N> {
    fn iter_data(&self) -> IterData<'_, T, N> {
        IterData {
            indices: self.indices,
            data: self.data,
            length: self.indices.len(),
            index: 0,
            depth: self.depth,
        }
    }

    fn iter(&self) -> IterRef<'_, T, N> {
        if self.depth + 1 >= N {
            panic!("Unable to iterate deeper than current depth {}", self.depth);
        }

        IterRef {
            indices: self.indices,
            data: self.data,
            depth: self.depth,
            length: self.indices.len(),
            index: 0,
        }
    }

    fn iter_flatten(&self) -> IterFlatten<'_, T, N> {
        IterFlatten {
            indices: self.indices,
            data: self.data,
            length: self.indices.len(),
            index: 0,
        }
    }
}

pub struct IterRef<'a, T, const N: usize> {
    indices: &'a [[usize; N]],
    data: &'a [T],
    depth: usize,
    length: usize,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for IterRef<'a, T, N> {
    type Item = RefInterVec<'a, T, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }
        let depth_index = self.indices[self.index][self.depth];
        let start = self.index;
        self.index += 1;

        while self.index < self.length && self.indices[self.index][self.depth] == depth_index {
            self.index += 1;
        }
        let end = self.index;
        let length = if self.depth + 1 <= N {
            self.indices[self.index - 1][self.depth + 1] + 1
        } else {
            1
        };
        Some(RefInterVec {
            indices: &self.indices[start..end],
            data: &self.data[start..end],
            length,
            depth: self.depth + 1,
        })
    }
}

pub struct IterFlatten<'a, T, const N: usize> {
    indices: &'a [[usize; N]],
    data: &'a [T],
    length: usize,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for IterFlatten<'a, T, N> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length {
            return None;
        }
        let (depth_index, depth_indices) = self.indices[self.index].split_last().unwrap();
        let start = self.index;
        self.index += 1;

        while self.index < self.length {
            let (next_index, next_indices) = self.indices[self.index].split_last().unwrap();
            if next_index != depth_index && next_indices == depth_indices {
                self.index += 1;
            } else {
                break;
            }
        }
        let end = self.index;
        Some(&self.data[start..end])
    }
}

impl<T, const N: usize> InterVec<T, N> {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            data: Vec::new(),
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<RefInterVec<'_, T, N>> {
        if index >= self.length {
            return None;
        }

        let mut start = 0;
        while start < self.indices.len() && self.indices[start][0] != index {
            start += 1;
        }
        let mut end = start;
        while end < self.indices.len() && self.indices[end][0] == index {
            end += 1;
        }

        let length = self.indices[end - 1][1] + 1;
        Some(RefInterVec {
            indices: &self.indices[start..end],
            data: &self.data[start..end],
            length,
            depth: 1,
        })
    }

    pub fn push(&mut self, value: T, indices: [usize; N]) {
        self.length = self.length.max(indices[0] + 1);
        self.data.push(value);
        self.indices.push(indices);
    }

    pub fn extend<const M: usize>(&mut self, other: InterVec<T, M>) {
        if M > N {
            panic!(
                "Unable to extend the current structure with a deeper \
                another ({M} (other structure) > {N} (current structure))"
            );
        } else if M == N {
            self.indices.extend(
                other
                    .indices
                    .iter()
                    .map(|indices| -> [usize; N] {
                        from_fn(|i| {
                            if i == 0 {
                                indices[i] + self.length
                            } else {
                                indices[i]
                            }
                        })
                    })
                    .collect::<Vec<[usize; N]>>(),
            );
            self.data.extend(other.data);
            self.length += other.length;
        } else {
            self.indices.extend(
                other
                    .indices
                    .iter()
                    .map(|indices| -> [usize; N] {
                        from_fn(|i| {
                            if i == 0 {
                                self.length
                            } else if N - i <= M {
                                indices[M + i - N]
                            } else {
                                0
                            }
                        })
                    })
                    .collect::<Vec<[usize; N]>>(),
            );
            self.data.extend(other.data);
            self.length += 1;
        }
    }
}

impl<'a, T, const N: usize> InterIterator<'a, T, N> for InterVec<T, N> {
    fn iter(&self) -> IterRef<'_, T, N> {
        if N == 0 {
            panic!("Unable to iterate deeper than current depth 0");
        }
        IterRef {
            indices: self.indices.as_slice(),
            data: self.data.as_slice(),
            depth: 0,
            length: self.indices.len(),
            index: 0,
        }
    }

    fn iter_data(&self) -> IterData<'_, T, N> {
        IterData {
            indices: self.indices.as_slice(),
            data: self.data.as_slice(),
            length: self.indices.len(),
            index: 0,
            depth: 0,
        }
    }

    fn iter_flatten(&self) -> IterFlatten<'_, T, N> {
        IterFlatten {
            indices: self.indices.as_slice(),
            data: self.data.as_slice(),
            length: self.indices.len(),
            index: 0,
        }
    }
}

impl<T, const N: usize> FromIterator<([usize; N], T)> for InterVec<T, N> {
    fn from_iter<I: IntoIterator<Item = ([usize; N], T)>>(iter: I) -> Self {
        let mut length = 0;
        let (indices, data) = iter
            .into_iter()
            .map(|(indices, data)| {
                length = length.max(indices[0] + 1);
                (indices, data)
            })
            .unzip();
        Self {
            indices,
            data,
            length,
        }
    }
}

impl<T, const N: usize, const M: usize> FromIterator<InterVec<T, M>> for InterVec<T, N> {
    fn from_iter<I: IntoIterator<Item = InterVec<T, M>>>(iter: I) -> Self {
        let mut length = 0;
        let mut indices = Vec::new();
        let mut data = Vec::new();
        for (k, item) in iter.into_iter().enumerate() {
            length += 1;
            indices.extend(item.indices.iter().map(|indices| -> [usize; N] {
                from_fn(|i| if i == 0 { k } else { indices[i - 1] })
            }));
            data.extend(item.data);
        }
        Self {
            indices,
            data,
            length,
        }
    }
}

impl<T> FromIterator<Vec<T>> for InterVec<T, 2> {
    fn from_iter<I: IntoIterator<Item = Vec<T>>>(iter: I) -> Self {
        let mut data = Vec::new();
        let mut indices = Vec::new();
        let mut length = 0;
        for (i, item) in iter.into_iter().enumerate() {
            length += 1;
            indices.extend((0..item.len()).map(|k| [i, k]));
            data.extend(item);
        }
        Self {
            indices,
            data,
            length,
        }
    }
}

impl<T> FromIterator<Vec<Vec<T>>> for InterVec<T, 3> {
    fn from_iter<I: IntoIterator<Item = Vec<Vec<T>>>>(iter: I) -> Self {
        let mut data = Vec::new();
        let mut indices = Vec::new();
        let mut length = 0;
        for (i, item) in iter.into_iter().enumerate() {
            length += 1;
            for (j, item) in item.into_iter().enumerate() {
                indices.extend((0..item.len()).map(|k| [i, j, k]));
                data.extend(item);
            }
        }
        Self {
            indices,
            data,
            length,
        }
    }
}

impl<'py, T: Clone + IntoPyObject<'py>> IntoPyObject<'py> for InterVec<T, 2> {
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let list = PyList::empty(py);
        for data in self.iter_data() {
            let values = PyList::new(py, data.to_vec())?;
            list.append(values)?;
        }
        Ok(list)
    }
}

impl<'py, T: Clone + IntoPyObject<'py>> IntoPyObject<'py> for InterVec<T, 3> {
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let list = PyList::empty(py);
        for ref_vec in self.iter() {
            let container = PyList::empty(py);
            for data in ref_vec.iter_data() {
                let values = PyList::new(py, data.to_vec())?;
                container.append(values)?;
            }
            list.append(container)?;
        }
        Ok(list)
    }
}

impl<'py, T: Clone + IntoPyObject<'py>> IntoPyObject<'py> for &InterVec<T, 2> {
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let list = PyList::empty(py);
        for data in self.iter_data() {
            let values = PyList::new(py, data.to_vec())?;
            list.append(values)?;
        }
        Ok(list)
    }
}

impl<'py, T: Clone + IntoPyObject<'py>> IntoPyObject<'py> for &InterVec<T, 3> {
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let list = PyList::empty(py);
        for ref_vec in self.iter() {
            let container = PyList::empty(py);
            for data in ref_vec.iter_data() {
                let values = PyList::new(py, data.to_vec())?;
                container.append(values)?;
            }
            list.append(container)?;
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_test() -> (Vec<Vec<Vec<Vec<i32>>>>, InterVec<i32, 4>) {
        let input = vec![
            vec![
                vec![vec![3007]],
                vec![vec![3008, 4002]],
                vec![vec![3009]],
                vec![vec![3010], vec![4003]],
                vec![vec![3011, 3012, 3013, 3014, 3015, 3016, 3017, 3018]],
            ],
            vec![
                vec![vec![3007]],
                vec![vec![3008, 4002]],
                vec![vec![3009]],
                vec![vec![3010], vec![4003]],
                vec![vec![3011, 3012, 3013, 3014, 3015, 3016, 3017, 3018]],
            ],
        ];
        let mut output = InterVec::new();
        for (i, level1) in input.iter().enumerate() {
            for (j, level2) in level1.iter().enumerate() {
                for (k, level3) in level2.iter().enumerate() {
                    for (l, &value) in level3.iter().enumerate() {
                        output.push(value, [i, j, k, l]);
                    }
                }
            }
        }
        (input, output)
    }

    #[test]
    fn test_init() {
        let (input, output) = input_test();
        assert_eq!(input.len(), output.len());
    }

    #[test]
    fn test_iter() {
        let (input, output) = input_test();
        let refs: Vec<RefInterVec<'_, i32, 4>> = output.iter().collect();
        assert_eq!(refs.len(), input.len());
        for (i, value) in refs.iter().enumerate() {
            assert_eq!(
                value.data,
                input[i]
                    .iter()
                    .flatten()
                    .flatten()
                    .map(|x| *x)
                    .collect::<Vec<i32>>()
            );
            assert_eq!(value.len(), input[i].len());
            assert_eq!(value.indices, {
                let mut indices = Vec::new();
                for (j, level2) in input[i].iter().enumerate() {
                    for (k, level3) in level2.iter().enumerate() {
                        for l in 0..level3.len() {
                            indices.push([i, j, k, l]);
                        }
                    }
                }
                indices
            });
        }
    }

    #[test]
    fn test_iter_data() {
        let (input, output) = input_test();
        let actual: Vec<&[i32]> = output.iter_data().collect();
        let expected: Vec<Vec<i32>> = input
            .iter()
            .map(|value| value.iter().flatten().flatten().map(|x| *x).collect())
            .collect();
        assert_eq!(actual.len(), expected.len());
        for i in 0..actual.len() {
            assert_eq!(actual[i], expected[i]);
        }
    }

    #[test]
    fn test_iter_flatten() {
        let (input, output) = input_test();
        let actual: Vec<&[i32]> = output.iter_flatten().collect();
        let expected: Vec<&Vec<i32>> = input.iter().flatten().flatten().collect();
        assert_eq!(actual.len(), expected.len());
        for i in 0..actual.len() {
            assert_eq!(actual[i], expected[i]);
        }
    }

    #[test]
    fn test_ref_iter() {
        let (input, output) = input_test();
        for (i, ref_inter_vec) in output.iter().enumerate() {
            let actual: Vec<RefInterVec<'_, i32, 4>> = ref_inter_vec.iter().collect();
            let expected = &input[i];
            assert_eq!(actual.len(), expected.len());
            for (j, value) in actual.iter().enumerate() {
                assert_eq!(
                    value.data,
                    expected[j]
                        .iter()
                        .flatten()
                        .map(|x| *x)
                        .collect::<Vec<i32>>()
                );
                assert_eq!(value.len(), expected[j].len());
                assert_eq!(value.indices, {
                    let mut indices = Vec::new();
                    for (k, level3) in expected[j].iter().enumerate() {
                        for l in 0..level3.len() {
                            indices.push([i, j, k, l]);
                        }
                    }
                    indices
                });
            }
        }
    }

    #[test]
    fn test_ref_iter_data() {
        let (input, output) = input_test();
        let actual: Vec<Vec<Vec<i32>>> = output
            .iter()
            .map(|values| {
                values
                    .iter_data()
                    .map(|x| x.iter().map(|&x| x).collect())
                    .collect()
            })
            .collect();
        let expected: Vec<Vec<Vec<i32>>> = input
            .iter()
            .map(|value| {
                value
                    .iter()
                    .map(|value| value.iter().flatten().map(|x| *x).collect())
                    .collect()
            })
            .collect();
        assert_eq!(actual.len(), expected.len());
        for i in 0..actual.len() {
            assert_eq!(actual[i].len(), expected[i].len());
            for j in 0..actual[i].len() {
                assert_eq!(actual[i][j], expected[i][j]);
            }
        }
    }

    #[test]
    fn test_ref_iter_flatten() {
        let (input, output) = input_test();
        let actual: Vec<Vec<Vec<i32>>> = output
            .iter()
            .map(|values| values.iter_flatten().map(|x| x.to_vec()).collect())
            .collect();
        let expected: Vec<Vec<Vec<i32>>> = input
            .iter()
            .map(|value| {
                value
                    .iter()
                    .flatten()
                    .map(|value| value.iter().map(|x| *x).collect())
                    .collect()
            })
            .collect();
        assert_eq!(actual.len(), expected.len());
        for i in 0..actual.len() {
            assert_eq!(actual[i].len(), expected[i].len());
            for j in 0..actual[i].len() {
                assert_eq!(actual[i][j], expected[i][j]);
            }
        }
    }

    #[test]
    fn test_extend_small_dim_case() {
        let (_, mut output) = input_test();
        let other = InterVec::from_iter([([0, 0], 10), ([1, 0], 20), ([1, 1], 20)]);
        let iv_length = output.len();
        let data_length = output.data.len();
        let indices_length = output.indices.len();
        output.extend(other);
        assert_eq!(output.len(), iv_length + 1);
        assert_eq!(output.data.len(), data_length + 3);
        assert_eq!(output.indices.len(), indices_length + 3);
        assert_eq!(
            output.indices[indices_length..],
            [[2, 0, 0, 0], [2, 0, 1, 0], [2, 0, 1, 1]]
        );
    }

    #[test]
    fn test_extend_same_dim_case() {
        let (_, mut output) = input_test();
        let other =
            InterVec::from_iter([([0, 0, 0, 0], 10), ([1, 0, 0, 0], 20), ([1, 0, 0, 1], 20)]);
        let iv_length = output.len();
        let data_length = output.data.len();
        let indices_length = output.indices.len();
        output.extend(other);
        assert_eq!(output.len(), iv_length + 2);
        assert_eq!(output.data.len(), data_length + 3);
        assert_eq!(output.indices.len(), indices_length + 3);
        assert_eq!(
            output.indices[indices_length..],
            [[2, 0, 0, 0], [3, 0, 0, 0], [3, 0, 0, 1]]
        );
    }

    #[test]
    fn test_convert_last_layer_into_vec() {
        let (input, output) = input_test();
        for (i, level1) in output.iter().enumerate() {
            for (j, level2) in level1.iter().enumerate() {
                for (k, arcs) in level2.iter_data().enumerate() {
                    let actual: Vec<f64> = arcs.iter().map(|&x| x as f64).collect();
                    let expected: Vec<f64> = input[i][j][k].iter().map(|&x| x as f64).collect();
                    assert_eq!(actual, expected);
                }
            }
        }
    }

    #[test]
    fn test_get() {
        let (input, output) = input_test();
        let value = output.get(1).unwrap();
        assert_eq!(
            value.data,
            input[1]
                .iter()
                .flatten()
                .flatten()
                .map(|x| *x)
                .collect::<Vec<i32>>()
        );
        assert_eq!(value.len(), input[1].len());
        assert_eq!(value.indices, {
            let mut indices = Vec::new();
            for (j, level2) in input[1].iter().enumerate() {
                for (k, level3) in level2.iter().enumerate() {
                    for l in 0..level3.len() {
                        indices.push([1, j, k, l]);
                    }
                }
            }
            indices
        });
    }

    #[test]
    fn test_ref_get() {
        let (input, output) = input_test();
        let value = output.get(1).unwrap();
        let value = value.get(1).unwrap();
        let i = 1;
        let j = 1;
        assert_eq!(
            value.data,
            input[i][j]
                .iter()
                .flatten()
                .map(|x| *x)
                .collect::<Vec<i32>>()
        );
        assert_eq!(value.len(), input[i][j].len());
        assert_eq!(value.indices, {
            let mut indices = Vec::new();
            for (k, level3) in input[i][j].iter().enumerate() {
                for l in 0..level3.len() {
                    indices.push([i, j, k, l]);
                }
            }
            indices
        });
    }

    #[test]
    fn test_from_inter_vec() {
        let (input, expected) = input_test();
        let mut collection = Vec::new();
        for level1 in input.iter() {
            let mut inter_vec = InterVec::new();
            for (j, level2) in level1.iter().enumerate() {
                for (k, level3) in level2.iter().enumerate() {
                    for (l, &value) in level3.iter().enumerate() {
                        inter_vec.push(value, [j, k, l]);
                    }
                }
            }
            collection.push(inter_vec);
        }
        let actual: InterVec<i32, 4> = InterVec::from_iter(collection);
        assert_eq!(actual.data, expected.data);
        assert_eq!(actual.indices, expected.indices);
        assert_eq!(actual.length, expected.length);
    }

    #[test]
    fn test_from_vec() {
        let values = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let input = InterVec::from_iter(values);
        assert_eq!(input.data, [0, 1, 2, 3, 4, 5]);
        assert_eq!(
            input.indices,
            [[0, 0], [0, 1], [0, 2], [1, 0], [1, 1], [1, 2]]
        );
        assert_eq!(input.length, 2);
    }
}
