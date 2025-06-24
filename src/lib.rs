#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone)]
pub struct Pos3D(pub f64, pub f64, pub f64);

#[derive(Debug, Copy, Clone)]
pub struct Bounds(f64, f64);

#[derive(Debug, Copy, Clone)]
pub struct Bounds3D {
    pub x: Bounds,
    pub y: Bounds,
    pub z: Bounds,
}

fn sq(x: f64) -> f64 {
    x * x
}
pub fn distance(a: Pos3D, b: Pos3D) -> f64 {
    let Pos3D(p_x, p_y, p_z) = a;
    let Pos3D(t_x, t_y, t_z) = b;
    (sq(t_x - p_x) + sq(t_y - p_y) + sq(t_z - p_z)).sqrt()
}
pub fn length(v: Pos3D) -> f64 {
    distance(v, Pos3D(0.0, 0.0, 0.0))
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Side {
    Pos,
    Neg,
}

pub type OctSide = (Side, Side, Side);

#[derive(Debug)]
pub enum TreeContents<P, T> {
    /// Up to 8 partitions holding the data
    Split(HashMap<OctSide, BoundedOctTree<P, T>>),
    /// Points in this area
    Whole(Vec<(P, T)>),
}
#[derive(Debug)]
pub struct BoundedOctTree<P, T> {
    /// The grounds that this tree covers
    bounds: Bounds3D,
    /// How many objects to hold before splitting
    count_threshold: usize,
    /// How many objects are in this oct tree
    count: usize,
    /// What's in the tree
    tree_contents: TreeContents<P, T>,
}

// goals:
// - query closest object from position
// - insert new objects
// - update positions of objects and re-balance tree

impl<P, T> BoundedOctTree<P, T>
where
    P: Clone + Into<Pos3D> + Debug,
    T: Debug,
{
    pub fn new(bounds: Bounds3D, count_threshold: usize) -> Self {
        if count_threshold == 0 {
            panic!("Cannot create an BoundedOctTree with a count threshold of 0")
        }
        BoundedOctTree {
            bounds,
            count_threshold,
            count: 0,
            tree_contents: TreeContents::Whole(vec![]),
        }
    }

    pub fn insert(&mut self, position: P, object: T) {
        self.count += 1;
        let split_map = match &mut self.tree_contents {
            TreeContents::Split(map) => {
                let side = self.bounds.get_oct_side(position.clone().into());
                if let Some(bot) = map.get_mut(&side) {
                    bot.insert(position, object);
                } else {
                    let mut bot = BoundedOctTree::new(
                        self.bounds.get_side_partition(side),
                        self.count_threshold,
                    );
                    bot.insert(position, object);
                    map.insert(side, bot);
                }
                return;
            }
            TreeContents::Whole(objs) => {
                objs.push((position, object));
                if objs.len() > self.count_threshold {
                    // we have to do a split
                    let mut split_map: HashMap<OctSide, BoundedOctTree<P, T>> = HashMap::new();
                    for (p, t) in objs.drain(..) {
                        let oct_side = self.bounds.get_oct_side(p.clone().into());
                        if let Some(bot) = split_map.get_mut(&oct_side) {
                            bot.insert(p, t);
                        } else {
                            let mut bot = BoundedOctTree::new(
                                self.bounds.get_side_partition(oct_side),
                                self.count_threshold,
                            );
                            bot.insert(p, t);
                            split_map.insert(oct_side, bot);
                        }
                    }
                    split_map
                } else {
                    return;
                }
            }
        };
        self.tree_contents = TreeContents::Split(split_map);
    }

    pub fn insert_all(&mut self, objs: impl Iterator<Item = (P, T)>) {
        for (position, obj) in objs {
            self.insert(position, obj);
        }
    }

    pub fn query_closest(&self, query_pos: &P) -> Option<(f64, &(P, T))> {
        let qpos = query_pos.clone().into();
        match &self.tree_contents {
            TreeContents::Split(partitions) => {
                let oct_side = self.bounds.get_oct_side(query_pos.clone().into());
                let mut min_dist = None;
                let mut result = None;
                if let Some(inner) = partitions.get(&oct_side) {
                    if let Some((dist, inner_result)) = inner.query_closest(query_pos) {
                        min_dist = Some(dist);
                        result = Some(inner_result);
                    }
                }
                // whether or not there was an inner result, we have to check the other bounds
                for (side, bot) in partitions.iter() {
                    if side == &oct_side {
                        continue;
                    }
                    let dist_to = bot.bounds.distance(qpos);
                    if min_dist.map(|d| dist_to < d).unwrap_or(true) {
                        if let Some((inner_dist, inner_result)) = bot.query_closest(&query_pos) {
                            if let Some(md) = min_dist {
                                if inner_dist < md {
                                    min_dist = Some(inner_dist);
                                    result = Some(inner_result);
                                }
                            } else {
                                min_dist = Some(inner_dist);
                                result = Some(inner_result);
                            }
                        }
                    }
                }
                match (min_dist, result) {
                    (Some(min_dist), Some(result)) => Some((min_dist, result)),
                    _ => None,
                }
            }
            TreeContents::Whole(objs) => {
                if objs.is_empty() {
                    None
                } else {
                    let mut md = distance(objs[0].0.clone().into(), qpos);
                    let mut res = &objs[0];
                    for obj in objs[1..].iter() {
                        let p = &obj.0;
                        let d = distance(p.clone().into(), qpos);
                        if d < md {
                            md = d;
                            res = obj;
                        }
                    }
                    Some((md, res))
                }
            }
        }
    }

    pub fn query_within(&self, query_pos: &P, radius: f64) -> Vec<&(P, T)> {
        let qpos = query_pos.clone().into();
        let mut output = vec![];
        match &self.tree_contents {
            TreeContents::Split(partitions) => {
                // we should check all the sections
                for (_side, bot) in partitions.iter() {
                    let dist_to = bot.bounds.distance(qpos);
                    if dist_to <= radius {
                        let results = bot.query_within(&query_pos, radius);
                        output.extend(results);
                    }
                }
            }
            TreeContents::Whole(objs) => {
                for obj in objs[1..].iter() {
                    let p = &obj.0;
                    let d = distance(p.clone().into(), qpos);
                    if d < radius {
                        output.push(obj);
                    }
                }
            }
        }
        output
    }
}

impl Bounds3D {
    pub fn get_oct_side(&self, p: Pos3D) -> OctSide {
        let Pos3D(x, y, z) = p;
        (self.x.get_side(x), self.y.get_side(y), self.z.get_side(z))
    }
    pub fn get_side_partition(&self, oct_side: OctSide) -> Bounds3D {
        let mid_x = self.x.mid();
        let mid_y = self.y.mid();
        let mid_z = self.z.mid();
        let (sx, sy, sz) = oct_side;
        Bounds3D {
            x: sx.choose(Bounds(self.x.0, mid_x), Bounds(mid_x, self.x.1)),
            y: sy.choose(Bounds(self.y.0, mid_y), Bounds(mid_y, self.y.1)),
            z: sz.choose(Bounds(self.z.0, mid_z), Bounds(mid_z, self.z.1)),
        }
    }
    pub fn mid(&self) -> Pos3D {
        Pos3D(self.x.mid(), self.y.mid(), self.z.mid())
    }
    pub fn size(&self) -> Pos3D {
        Pos3D(self.x.size(), self.y.size(), self.z.size())
    }
    /// Returns a negative value if it is inside the bounds
    pub fn distance(&self, wp: Pos3D) -> f64 {
        let Pos3D(mx, my, mz) = self.mid();
        let Pos3D(wx, wy, wz) = wp;
        let Pos3D(bx, by, bz) = self.size();
        let px = wx - mx;
        let py = wy - my;
        let pz = wz - mz;
        let qx = px.abs() - bx / 2.;
        let qy = py.abs() - by / 2.;
        let qz = pz.abs() - bz / 2.;
        let lq = Pos3D(qx.max(0.0), qy.max(0.0), qz.max(0.0));
        length(lq) + f64::min(f64::max(qx, f64::max(qy, qz)), 0.0)
    }
}

impl Bounds {
    pub fn get_side(&self, p: f64) -> Side {
        if p > self.mid() { Side::Pos } else { Side::Neg }
    }
    pub fn mid(&self) -> f64 {
        (self.0 + self.1) / 2.
    }
    pub fn size(&self) -> f64 {
        self.1 - self.0
    }
}

impl Side {
    fn choose<T>(&self, neg: T, pos: T) -> T {
        match &self {
            Side::Pos => pos,
            Side::Neg => neg,
        }
    }
}
