use std::collections::HashMap;

use crate::bisect::bisect;
use crate::topojson_structs::{Geometry, GeometryType};

pub fn wrap_neighbors(objects: &[Geometry]) -> Vec<Vec<i32>> {
    Neighbors::call(objects)
}

struct Neighbors {
    indexes_by_arc: HashMap<usize, Vec<usize>>,
    neighbors: Vec<Vec<i32>>,
}

impl Neighbors {
    fn call(objects: &[Geometry]) -> Vec<Vec<i32>> {
        Neighbors::new(objects.len()).neighbors(objects)
    }

    fn neighbors(mut self, objects: &[Geometry]) -> Vec<Vec<i32>> {
        objects
            .iter()
            .enumerate()
            .for_each(|(i, o)| self.geometry(o, i));

        let mut splice_neighbors = |i1: usize, i2: usize| {
            let n = &mut self.neighbors[i1];
            let i2 = i2 as i32;
            let i = bisect(n, &i2);
            match n.get(i) {
                Some(&neighbor) => {
                    if neighbor != i2 {
                        n.insert(i, i2);
                    }
                }
                None => n.insert(i, i2),
            }
        };

        for indexes in self.indexes_by_arc.values() {
            let m = indexes.len();
            for j in 0..m {
                for k in (j + 1)..m {
                    let ij = indexes[j];
                    let ik = indexes[k];
                    splice_neighbors(ij, ik);
                    splice_neighbors(ik, ij);
                }
            }
        }

        self.neighbors
    }

    fn new(len_objects: usize) -> Self {
        Self {
            indexes_by_arc: HashMap::new(),
            neighbors: vec![Vec::new(); len_objects],
        }
    }

    fn line(&mut self, arcs: &[i32], i: usize) {
        arcs.iter().for_each(|&a| {
            let a = if a < 0 { !a } else { a } as usize;
            self.indexes_by_arc
                .entry(a)
                .and_modify(|o| o.push(i))
                .or_insert(vec![i]);
        });
    }

    fn polygon(&mut self, arcs: &[Vec<i32>], i: usize) {
        arcs.iter().for_each(|arc| self.line(arc, i));
    }

    fn geometry(&mut self, o: &Geometry, i: usize) {
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                geometries.iter().for_each(|o| self.geometry(o, i))
            }
            GeometryType::LineString { arcs } => self.line(arcs, i),
            GeometryType::MultiLineString { arcs } => self.polygon(arcs, i),
            GeometryType::Polygon { arcs } => self.polygon(arcs, i),
            GeometryType::MultiPolygon { arcs } => arcs.iter().for_each(|arc| self.polygon(arc, i)),
            _ => panic!("Invalid geometry type used during neighbors operation"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbors_1() {
        assert_eq!(wrap_neighbors(&Vec::new()), Vec::<Vec<i32>>::new());
    }

    //
    // A-----B
    //           or   A-----B-----C
    // C-----D
    //
    #[test]
    fn test_neighbors_2() {
        let objects: Vec<Geometry> = [vec![0], vec![1]]
            .map(|arcs| Geometry {
                geometry: GeometryType::LineString { arcs },
                id: None,
                properties: None,
                bbox: None,
            })
            .into_iter()
            .collect();
        assert_eq!(wrap_neighbors(&objects), vec![Vec::<i32>::new(); 2]);
    }

    //
    // A-----B-----C-----D
    //
    #[test]
    fn test_neighbors_3() {
        let objects: Vec<Geometry> = [vec![0, 1], vec![1, 2]]
            .map(|arcs| Geometry {
                geometry: GeometryType::LineString { arcs },
                id: None,
                properties: None,
                bbox: None,
            })
            .into_iter()
            .collect();
        assert_eq!(wrap_neighbors(&objects), vec![vec![1], vec![0]]);
    }

    //
    // A-----B-----C-----D
    //
    #[test]
    fn test_neighbors_4() {
        let objects: Vec<Geometry> = [vec![0, 1], vec![2, -2]]
            .map(|arcs| Geometry {
                geometry: GeometryType::LineString { arcs },
                id: None,
                properties: None,
                bbox: None,
            })
            .into_iter()
            .collect();
        assert_eq!(wrap_neighbors(&objects), vec![vec![1], vec![0]]);
    }

    //
    // A-----B-----C-----D-----E-----F
    //
    #[test]
    fn test_neighbors_5() {
        let objects: Vec<Geometry> = [
            vec![0, 1, 2],
            vec![1, 2, 3],
            vec![2, 3, 4],
            vec![-3, -2, -1],
            vec![-4, -3, -2],
            vec![-5, -4, -3],
        ]
        .map(|arcs| Geometry {
            geometry: GeometryType::LineString { arcs },
            id: None,
            properties: None,
            bbox: None,
        })
        .into_iter()
        .collect();
        assert_eq!(
            wrap_neighbors(&objects),
            vec![
                vec![1, 2, 3, 4, 5],
                vec![0, 2, 3, 4, 5],
                vec![0, 1, 3, 4, 5],
                vec![0, 1, 2, 4, 5],
                vec![0, 1, 2, 3, 5],
                vec![0, 1, 2, 3, 4]
            ]
        );
    }

    //
    // A-----B-----E     G
    // |     |     |     |\
    // |     |     |     | \
    // |     |     |     |  \
    // |     |     |     |   \
    // |     |     |     |    \
    // D-----C-----F     I-----H
    //
    #[test]
    fn test_neighbors_6() {
        let objects: Vec<Geometry> = [vec![vec![0, 1]], vec![vec![2, -1]], vec![vec![3]]]
            .map(|arcs| Geometry {
                geometry: GeometryType::Polygon { arcs },
                id: None,
                properties: None,
                bbox: None,
            })
            .into_iter()
            .collect();
        assert_eq!(wrap_neighbors(&objects), vec![vec![1], vec![0], vec![]]);
    }

    //
    // A-----------B-----------C
    // |           |           |
    // |           |           |
    // |     D-----E-----F     |
    // |     |           |     |
    // |     |           |     |
    // |     G-----H-----I     |
    // |           |           |
    // |           |           |
    // J-----------K-----------L
    //
    #[test]
    fn test_neighbors_7() {
        let objects: Vec<Geometry> = [vec![vec![0, 1, 2, 3]], vec![vec![4, -3, 5, -1]]]
            .map(|arcs| Geometry {
                geometry: GeometryType::Polygon { arcs },
                id: None,
                properties: None,
                bbox: None,
            })
            .into_iter()
            .collect();
        assert_eq!(wrap_neighbors(&objects), vec![vec![1], vec![0]]);
    }
}
