use indexmap::IndexMap;
use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::topojsons::TopoJSON;

pub fn stitch(topology: &TopoJSON, arcs: Vec<i32>) -> Vec<Vec<i32>> {
    Stitch::call(topology, arcs)
}

#[derive(Debug, PartialEq, Clone)]
struct Fragment {
    start: [i32; 2],
    end: [i32; 2],
    arcs: Vec<i32>,
}

impl Fragment {
    fn new(start: [i32; 2], end: [i32; 2], arcs: Vec<i32>) -> Self {
        Self { start, end, arcs }
    }

    fn push(&mut self, item: i32) {
        self.arcs.push(item);
    }

    fn unshift(&mut self, value: i32) {
        self.arcs.insert(0, value);
    }
}

trait AddFragments {
    fn add_fragment(&self, other: &Rc<RefCell<Fragment>>) -> Rc<RefCell<Fragment>>;
}

impl AddFragments for Rc<RefCell<Fragment>> {
    fn add_fragment(&self, other: &Rc<RefCell<Fragment>>) -> Rc<RefCell<Fragment>> {
        Rc::new(RefCell::new(Fragment {
            start: self.borrow().start,
            end: other.borrow().end,
            arcs: [&self.borrow().arcs[..], &other.borrow().arcs[..]].concat(),
        }))
    }
}

#[derive(Default)]
struct Stitch {
    stitched_arcs: HashSet<usize>,
    fragment_by_start: IndexMap<[i32; 2], Rc<RefCell<Fragment>>>,
    fragment_by_end: IndexMap<[i32; 2], Rc<RefCell<Fragment>>>,
    fragments: Vec<Vec<i32>>,
}

#[inline]
fn arc_index(i: i32) -> usize {
    (if i < 0 { !i } else { i }) as usize
}

impl Stitch {
    fn call(topology: &TopoJSON, arcs: Vec<i32>) -> Vec<Vec<i32>> {
        Self::default().fragments(topology, arcs)
    }

    fn replace(&mut self, fragment: Rc<RefCell<Fragment>>) {
        let start = fragment.borrow().start;
        let end = fragment.borrow().end;
        self.fragment_by_start
            .entry(start)
            .and_modify(|v| *v = fragment.clone())
            .or_insert(fragment.clone());
        self.fragment_by_end
            .entry(end)
            .and_modify(|v| *v = fragment.clone())
            .or_insert(fragment.clone());
    }

    fn replace_from(&mut self, fg: Rc<RefCell<Fragment>>, start: [i32; 2], end: [i32; 2]) {
        {
            let mut fg_borrow = fg.borrow_mut();
            fg_borrow.start = start;
            fg_borrow.end = end;
        }
        self.replace(fg);
    }

    fn fragments(mut self, topology: &TopoJSON, mut arcs: Vec<i32>) -> Vec<Vec<i32>> {
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
                let r = (j, empty_index, i);
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
            let (start, end) = self.ends(topology, i);

            if let Some(f) = self.fragment_by_end.get(&start).cloned() {
                self.fragment_by_end.shift_remove(&f.borrow().end);
                {
                    f.borrow_mut().push(*i);
                    f.borrow_mut().end = end;
                }

                if let Some(g) = self.fragment_by_start.get(&end).cloned() {
                    self.fragment_by_start.shift_remove(&g.borrow().start);

                    let start = f.borrow().start;
                    let end = g.borrow().end;
                    let fg = if f == g {
                        f.clone()
                    } else {
                        f.add_fragment(&g)
                    };
                    self.replace_from(fg, start, end);
                } else {
                    self.replace(f.clone());
                }
            } else if let Some(f) = self.fragment_by_start.get(&end).cloned() {
                self.fragment_by_start.shift_remove(&f.borrow().start);
                {
                    f.borrow_mut().unshift(*i);
                    f.borrow_mut().start = start;
                }

                if let Some(g) = self.fragment_by_end.get(&start).cloned() {
                    self.fragment_by_end.shift_remove(&g.borrow().end);

                    let start = g.borrow().start;
                    let end = f.borrow().end;
                    let gf = if g == f {
                        f.clone()
                    } else {
                        g.add_fragment(&f)
                    };
                    self.replace_from(gf, start, end);
                } else {
                    self.replace(f.clone());
                }
            } else {
                self.replace(Rc::new(RefCell::new(Fragment::new(start, end, vec![*i]))));
            }
        }

        self.flush();

        arcs.iter()
            .filter(|&&i| !self.stitched_arcs.contains(&arc_index(i)))
            .for_each(|&i| self.fragments.push(vec![i]));
        self.fragments
    }

    fn ends(&self, topology: &TopoJSON, &i: &i32) -> ([i32; 2], [i32; 2]) {
        let arc = &topology.arcs[if i < 0 { !i as usize } else { i as usize }];
        let first = arc.first().unwrap();
        let p0: [i32; 2] = std::array::from_fn(|i| first[i]);

        let p1: [i32; 2] = if topology.transform.is_some() {
            [
                arc.iter().map(|x| x[0]).reduce(|a, b| a + b).unwrap(),
                arc.iter().map(|x| x[1]).reduce(|a, b| a + b).unwrap(),
            ]
        } else {
            let last = arc.last().unwrap();
            std::array::from_fn(|i| last[i])
        };
        if i < 0 { (p1, p0) } else { (p0, p1) }
    }

    fn flush(&mut self) {
        let mut banned_starts = HashSet::new();
        for (_, fragment) in self.fragment_by_end.drain(..) {
            let borrowed = fragment.borrow();
            banned_starts.insert(borrowed.start);
            self.stitched_arcs
                .extend(borrowed.arcs.iter().map(|&i| arc_index(i)));
            self.fragments.push(borrowed.arcs.clone());
        }

        for (_, fragment) in self.fragment_by_start.drain(..) {
            let borrowed = fragment.borrow();
            if banned_starts.contains(&borrowed.start) {
                continue;
            }
            self.stitched_arcs
                .extend(borrowed.arcs.iter().map(|&i| arc_index(i)));
            self.fragments.push(borrowed.arcs.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_fragment() {
        let f = Rc::new(RefCell::new(Fragment::new([1, 0], [2, 0], vec![1])));
        let f_same = Rc::new(RefCell::new(Fragment::new([1, 0], [2, 0], vec![1])));
        let g = Rc::new(RefCell::new(Fragment::new([1, 1], [2, 0], vec![1])));
        let g_clone = g.clone();
        assert!(f == f_same);
        assert!(f != g);
        assert!(g == g_clone);
    }

    #[test]
    fn test_push_fragment() {
        let f = Rc::new(RefCell::new(Fragment::new([1, 0], [2, 0], vec![1])));
        let g = f.clone();
        assert_eq!(g.borrow().arcs.len(), 1);
        {
            f.borrow_mut().push(1);
        }
        assert_eq!(g.borrow().arcs.len(), 2);
    }

    #[test]
    fn test_unshift_fragment() {
        let f = Rc::new(RefCell::new(Fragment::new([1, 0], [2, 0], vec![1])));
        let g = f.clone();
        assert_eq!(g.borrow().arcs.len(), 1);
        {
            let mut borrow_mut = f.borrow_mut();
            for i in 0..=10 {
                borrow_mut.unshift(i);
            }
        }
        assert_eq!(g.borrow().arcs.len(), 12);
        assert_eq!(g.borrow().arcs[0], 10);
    }
}
