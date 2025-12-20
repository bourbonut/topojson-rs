use std::{collections::HashMap, ops::Add, rc::Rc};

use crate::topojson_structs::TopoJSON;

pub fn wrap_stich(topology: TopoJSON, arcs: Vec<i32>) -> Vec<Vec<i32>> {
    Stitch::call(topology, arcs)
}

#[derive(PartialEq, Clone)]
struct Fragment {
    start: Rc<Vec<i32>>,
    end: Rc<Vec<i32>>,
    arcs: Rc<Vec<i32>>,
}

impl Fragment {
    fn new(start: Vec<i32>, end: Vec<i32>, arcs: Vec<i32>) -> Self {
        Self {
            start: Rc::new(start),
            end: Rc::new(end),
            arcs: Rc::new(arcs),
        }
    }

    fn push(&mut self, item: i32) {
        Rc::get_mut(&mut self.arcs).unwrap().push(item);
    }

    fn unshift(&mut self, value: i32) {
        Rc::get_mut(&mut self.arcs).unwrap().insert(0, value);
    }
}

// impl Add for Fragment {
//     type Output = Fragment;
//
//     fn add(self, rhs: Self) -> Self::Output {
//         Self {
//             start: self.start.clone(),
//             end: rhs.end.clone(),
//             arcs: Rc::new([&self.arcs[..], &rhs.arcs[..]].concat()),
//         }
//     }
// }

impl<'a, 'b> Add<&'b Fragment> for &'a Fragment {
    type Output = Fragment;

    fn add(self, other: &'b Fragment) -> Fragment {
        Fragment {
            start: self.start.clone(),
            end: other.end.clone(),
            arcs: Rc::new([&self.arcs[..], &other.arcs[..]].concat()),
        }
    }
}

#[derive(Default)]
struct Stitch {
    stitched_arcs: HashMap<usize, i32>,
    fragment_by_start: HashMap<Vec<i32>, Fragment>,
    fragment_by_end: HashMap<Vec<i32>, Fragment>,
    fragments: Vec<Vec<i32>>,
}

impl Stitch {
    fn call(topology: TopoJSON, arcs: Vec<i32>) -> Vec<Vec<i32>> {
        let mut stitch = Self::default();
        stitch.fragments(topology, arcs)
    }

    fn replace(&mut self, fragment: &Fragment) {
        self.fragment_by_start
            .get_mut(&fragment.start[..])
            .map(|v| *v = fragment.clone());
        self.fragment_by_end
            .get_mut(&fragment.end[..])
            .map(|v| *v = fragment.clone());
    }

    fn fragments(&mut self, topology: TopoJSON, mut arcs: Vec<i32>) -> Vec<Vec<i32>> {
        let mut empty_index: usize = 0;

        // Stitch empty arcs first, since they may be subsumed by other arcs.
        // j: index used for the `arcs` list
        // i: index used for the `topology["arcs"]` list
        let changes: Vec<(usize, usize, i32)> = arcs
            .iter()
            .enumerate()
            .filter(|&(_, &i)| {
                let arc = &topology.arcs[if i < 0 { !i as usize } else { i as usize }];
                arc.len() < 3 && arc[1][0] == 0 && arc[1][1] == 0
            })
            .map(|(j, &i)| {
                let r = (j, empty_index.clone(), i);
                empty_index += 1;
                r
            })
            .collect();

        for (j, index, i) in changes {
            let t = arcs[index];
            arcs[index] = i;
            arcs[j] = t;
        }

        for i in arcs.iter() {
            let (start, end) = self.ends(&topology, i);

            if let Some(f_end) = self
                .fragment_by_end
                .get(&start[..])
                .and_then(|f| Some(f.end.clone()))
            {
                self.fragment_by_end.get_mut(&start[..]).map(|f| {
                    f.push(*i);
                    f.end = Rc::new(end);
                });
                self.fragment_by_end.remove(&f_end[..]);

                let f = &self.fragment_by_end[&start[..]].clone();

                if let Some(g_start) = self
                    .fragment_by_start
                    .get(&f.end[..])
                    .and_then(|g| Some(g.start.clone()))
                {
                    self.fragment_by_start.remove(&g_start[..]);
                    let g = &self.fragment_by_start[&f.end[..]].clone();

                    let mut fg = if f == g { f.clone() } else { f + g };

                    fg.start = f.start.clone();
                    fg.end = g.end.clone();

                    self.replace(&fg);
                } else {
                    self.replace(&f);
                }
            } else if let Some(f_start) = self
                .fragment_by_start
                .get(&end[..])
                .and_then(|f| Some(f.start.clone()))
            {
                self.fragment_by_start.remove(&f_start[..]);
                self.fragment_by_start.get_mut(&end[..]).map(|f| {
                    f.unshift(*i);
                    f.start = Rc::new(start);
                });

                let f = &self.fragment_by_end[&end[..]].clone();

                if let Some(g_end) = self
                    .fragment_by_end
                    .get(&f.start[..])
                    .and_then(|g| Some(g.end.clone()))
                {
                    self.fragment_by_end.remove(&g_end[..]);
                    let g = &self.fragment_by_end[&f.start[..]].clone();

                    let mut gf = if f == g { f.clone() } else { g + f };

                    gf.start = g.start.clone();
                    gf.end = f.end.clone();

                    self.replace(&gf);
                } else {
                    self.replace(&f);
                }
            } else {
                let f = Fragment::new(start, end, vec![*i]);
                self.replace(&f);
            }
        }

        let mut fragments = Vec::new();
        for &i in arcs.iter() {
            if self
                .stitched_arcs
                .get(&(if i < 0 { !i } else { i } as usize))
                .is_none()
            {
                fragments.push(vec![i]);
            }
        }
        fragments
    }

    fn ends(&self, topology: &TopoJSON, &i: &i32) -> (Vec<i32>, Vec<i32>) {
        let arc = &topology.arcs[if i < 0 { !i as usize } else { i as usize }];
        let p0 = arc.first().unwrap().to_vec();

        let p1 = if topology.transform.is_some() {
            vec![
                arc.iter().map(|x| x[0]).reduce(|a, b| a + b).unwrap(),
                arc.iter().map(|x| x[1]).reduce(|a, b| a + b).unwrap(),
            ]
        } else {
            arc.last().unwrap().to_vec()
        };
        if i < 0 { (p1, p0) } else { (p0, p1) }
    }
}
