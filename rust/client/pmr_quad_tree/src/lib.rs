use common::DEFAULT_F64_MARGIN;
use float_cmp::ApproxEq;
use geometry::*;
use std::{
    borrow::Borrow,
    boxed::Box,
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    hash::Hash,
    mem,
    rc::{Rc, Weak},
};

// https://stackoverflow.com/questions/25903180/pmr-quadtree-data-structure-and-algorithm
#[derive(Clone, Debug)]
pub struct Config {
    pub initial_radius: f64,
    pub max_depth: u8,
    pub splitting_threshold: usize,
}

#[derive(Debug)]
pub struct Tree<K: Eq + Hash, S: Spatial<Hashed = K>> {
    pub config: Config,
    pub root: Node<S>,
    pub items: HashMap<K, Rc<S>>,
}

#[derive(Debug)]
pub enum NodeType<S: Spatial> {
    InnerNode(Box<InnerNode<S>>),
    Leaf(Leaf<S>),
}

#[derive(Debug)]
pub struct Node<S: Spatial> {
    node: NodeType<S>,
}

impl<S: Spatial> From<Leaf<S>> for Node<S> {
    fn from(leaf: Leaf<S>) -> Node<S> {
        Node {
            node: NodeType::Leaf(leaf)
        }
    }
}

impl<S: Spatial> From<InnerNode<S>> for Node<S> {
    fn from(inner_node: InnerNode<S>) -> Node<S> {
        Node {
            node: NodeType::InnerNode(Box::new(inner_node))
        }
    }
}

#[derive(Debug)]
pub struct Leaf<S: Spatial> {
    pub bounds: Bounds,
    pub items: Vec<Weak<S>>,
    pub level: u8,
}

#[derive(Debug)]
pub struct InnerNode<S: Spatial> {
    pub bounds: Bounds,
    pub level: u8,
    pub ne: Node<S>,
    pub nw: Node<S>,
    pub se: Node<S>,
    pub sw: Node<S>,
}

type BoundingLeaf<'b, S> = Option<&'b Leaf<S>>;

pub struct Neighbor<S: Spatial> {
    pub distance: f64,
    pub item: Weak<S>,
}

impl<S: Spatial> Leaf<S> {
    fn new(level: u8, bounds: Bounds) -> Leaf<S> {
        Leaf { level, bounds, items: Vec::new() }
    }

    fn is_full(&self, config: &Config) -> bool {
        if self.level == config.max_depth {
            return false
        }
        return self.items.len() > config.splitting_threshold
    }

    fn split(&self) -> InnerNode<S> {
        let split_bounds  = self.bounds.split();

        let mut leaves = [
            Leaf::new(self.level + 1, split_bounds.ne),
            Leaf::new(self.level + 1, split_bounds.nw),
            Leaf::new(self.level + 1, split_bounds.se),
            Leaf::new(self.level + 1, split_bounds.sw),
        ];

        for item in self.items.iter() {
            for leaf in leaves.iter_mut() {
                if let Some(rc_item) = item.upgrade() {
                    if rc_item.as_ref().borrow().intersects(&leaf.bounds) {
                        leaf.items.push(item.clone());
                    }
                }
            }
        }

        let [ne, nw, se, sw] = leaves;
        InnerNode {
            bounds: self.bounds.clone(),
            level: self.level,
            ne: Node { node: NodeType::Leaf(ne) },
            nw: Node { node: NodeType::Leaf(nw) },
            se: Node { node: NodeType::Leaf(se) },
            sw: Node { node: NodeType::Leaf(sw) },
        }
    }

    fn bounding_leaf<'b>(&'b self, point: &Point) -> BoundingLeaf<'b, S> {
        if !point.intersects(&self.bounds) {
            return None
        }
        Some(&self)
    }

    fn nearest_neighbor(&self, point: &Point) -> Option<Neighbor<S>> {
        let mut min_distance = std::f64::MAX;
        let mut arg_min: Option<Neighbor<S>> = None;
        for item in self.items.iter() {
            if let Some(rc_item) = item.upgrade() {
                let distance = rc_item.as_ref().borrow().distance(point);
                if distance < min_distance {
                    min_distance = distance;
                    arg_min = Some(Neighbor { distance, item: item.clone() });
                }
            }
        }
        arg_min
    }

    fn is_nearest_neighbor_candidate_leaf<'b>(&'b self, point: &Point, candidate_radius: f64, candidates: &mut Vec<&'b Leaf<S>>) {
        if self.bounds.distance(point) <= candidate_radius {
            candidates.push(&self);
        }
    }
}

impl<S: Spatial> InnerNode<S> {
    fn bounding_leaf<'b>(&'b self, point: &Point) -> BoundingLeaf<'b, S> {
        if !point.intersects(&self.bounds) {
            return None
        }
        for child in self.children().iter() {
            if let Some(leaf) = child.bounding_leaf(point) {
                return Some(leaf)
            }
        }
        None
    }

    fn children<'b>(&'b self) -> [&'b Node<S>; 4] {
        [&self.ne, &self.nw, &self.se, &self.sw]
    }

    fn find_nearest_neighbor_candidate_leaves<'b>(&'b self, point: &Point, candidate_radius: f64, candidates: &mut Vec<&'b Leaf<S>>) {
        for child in self.children().iter() {
            if child.bounds().distance(point) <= candidate_radius {
                child.find_nearest_neighbor_candidate_leaves(point, candidate_radius, candidates);
            }
        }
    }
}

impl<S: Spatial> Node<S> {
    fn bounds<'b>(&'b self) -> &'b Bounds {
        match &self.node {
            NodeType::InnerNode(inner_node) => &(*inner_node).bounds,
            NodeType::Leaf(leaf) => &leaf.bounds,
        }
    }

    fn bounding_leaf<'b>(&'b self, point: &Point) -> BoundingLeaf<'b, S> {
        match &self.node {
            NodeType::InnerNode(inner_node) => (*inner_node).bounding_leaf(point),
            NodeType::Leaf(leaf) => leaf.bounding_leaf(point),
        }
    }

    fn insert(&mut self, item: &Rc<S>, config: &Config) {
        if !item.as_ref().borrow().intersects(self.bounds()) { return }
        let mut new_node: Option<Node<S>> = None;
        match &mut self.node {
            NodeType::InnerNode(inner_node) => {
                (*inner_node).ne.insert(item, config);
                (*inner_node).nw.insert(item, config);
                (*inner_node).se.insert(item, config);
                (*inner_node).sw.insert(item, config);
            },
            NodeType::Leaf(leaf) => {
                leaf.items.push(Rc::downgrade(item));
                if leaf.is_full(config) {
                    new_node = Some(Node::from(leaf.split()));
                }
            },
        };
        if let Some(new_node) = new_node {
            self.node = new_node.node;
        }
    }

    fn find_nearest_neighbor_candidate_leaves<'b>(&'b self, point: &Point, candidate_radius: f64, candidates: &mut Vec<&'b Leaf<S>>) {
        match &self.node {
            NodeType::InnerNode(inner_node) => (*inner_node).find_nearest_neighbor_candidate_leaves(point, candidate_radius, candidates),
            NodeType::Leaf(leaf) => leaf.is_nearest_neighbor_candidate_leaf(point, candidate_radius, candidates),
        };
    }
}

impl<K: Eq + Hash, S: Spatial<Hashed = K>> Tree<K, S> {
    pub fn new(config: Config) -> Tree<K, S> {
        let radius = config.initial_radius;
        Tree {
            config,
            root: Node::from(Leaf::new(0, Bounds { center: ORIGIN, radius })),
            items: HashMap::new(),
        }
    }

    pub fn has(&self, key: &K) -> bool {
        match self.get(key) { Some(_) => true, None => false }
    }

    pub fn insert(&mut self, item: S) -> Option<Rc<S>> {
        let key = item.key(); // only call once, clones

        if let Some(item_rc) = self.get(&key) {
            return Some(item_rc)
        }

        // insert if intersects
        if item.intersects(self.root.bounds()) {
            let item = self.items.entry(key).or_insert(Rc::new(item));
            self.root.insert(item, &self.config);
            return None;
        }

        // otherwise, expand outward
        let root = mem::replace(&mut self.root, Node::from(Leaf::new(u8::MAX, Bounds { center: ORIGIN, radius: 0. })));

        let root_node = match root.node {
            NodeType::InnerNode(inner_node) => *inner_node,
            NodeType::Leaf(leaf) => leaf.split(),
        };

        let shifts = Shifts::new(2. * root_node.bounds.radius);

        let ne_bounds = root_node.ne.bounds();
        let nw_bounds = root_node.nw.bounds();
        let sw_bounds = root_node.sw.bounds();
        let se_bounds = root_node.se.bounds();

        // NE
        let ne_ne = Leaf::new(root_node.level + 1, ne_bounds.shift(&shifts.ne));
        let ne_nw = Leaf::new(root_node.level + 1, ne_bounds.shift(&shifts.n));
        let ne_se = Leaf::new(root_node.level + 1, ne_bounds.shift(&shifts.e));

        // NW
        let nw_ne = Leaf::new(root_node.level + 1, nw_bounds.shift(&shifts.n));
        let nw_nw = Leaf::new(root_node.level + 1, nw_bounds.shift(&shifts.nw));
        let nw_sw = Leaf::new(root_node.level + 1, nw_bounds.shift(&shifts.w));

        // SW
        let sw_nw = Leaf::new(root_node.level + 1, sw_bounds.shift(&shifts.w));
        let sw_sw = Leaf::new(root_node.level + 1, sw_bounds.shift(&shifts.sw));
        let sw_se = Leaf::new(root_node.level + 1, sw_bounds.shift(&shifts.s));

        // SE
        let se_ne = Leaf::new(root_node.level + 1, se_bounds.shift(&shifts.e));
        let se_sw = Leaf::new(root_node.level + 1, se_bounds.shift(&shifts.s));
        let se_se = Leaf::new(root_node.level + 1, se_bounds.shift(&shifts.se));

        self.root = Node::from(InnerNode {
            level: root_node.level - 1,
            bounds: root_node.bounds.mul(2.),
            ne: Node::from(InnerNode {
                level: root_node.level,
                bounds: ne_bounds.shift(&shifts.ne.mul(0.5)).mul(2.),
                ne: Node::from(ne_ne),
                nw: Node::from(ne_nw),
                se: Node::from(ne_se),
                sw: root_node.ne,
            }),
            nw: Node::from(InnerNode {
                level: root_node.level,
                bounds: nw_bounds.shift(&shifts.nw.mul(0.5)).mul(2.),
                ne: Node::from(nw_ne),
                nw: Node::from(nw_nw),
                se: root_node.nw,
                sw: Node::from(nw_sw),
            }),
            se: Node::from(InnerNode {
                level: root_node.level,
                bounds: sw_bounds.shift(&shifts.sw.mul(0.5)).mul(2.),
                ne: root_node.sw,
                nw: Node::from(sw_nw),
                se: Node::from(sw_se),
                sw: Node::from(sw_sw),
            }),
            sw: Node::from(InnerNode {
                level: root_node.level,
                bounds: se_bounds.shift(&shifts.se.mul(0.5)).mul(2.),
                ne: Node::from(se_ne),
                nw: root_node.se,
                se: Node::from(se_se),
                sw: Node::from(se_sw),
            }),
        });

        // try inserting again
        return self.insert(item);
    }

    pub fn get(&self, key: &K) -> Option<Rc<S>> {
        if let Some(item_rc) = self.items.get(&key) {
            return Some(item_rc.clone())
        }
        None
    }

    // https://www.cs.umd.edu/~hjs/pubs/ssd91.pdf - Section 5
    pub fn nearest_neighbor<'b>(&'b self, point: &Point) -> Result<Neighbor<S>, String> {
        let bounding_leaf = match self.root.bounding_leaf(point) {
            Some(leaf) => leaf,
            // this may or may not work - need to test out - kind of an approximation
            None => match self.root.bounding_leaf(&(point + &self.root.bounds().distance_vector(point))) {
                Some(leaf) => leaf,
                None => return Err(format!("no bounding leaf: {}", point + &self.root.bounds().distance_vector(point))),
            },
        };
        let mut candidate_leaves: Vec<&'b Leaf<S>> = vec![];
        let mut neighbors = BinaryHeap::new();

        self.root.find_nearest_neighbor_candidate_leaves(point, 4. * 2_f64.sqrt() * bounding_leaf.bounds.radius, &mut candidate_leaves);

        for leaf in candidate_leaves.into_iter() {
            if let Some(neighbor) = leaf.nearest_neighbor(point) {
                if neighbor.distance.approx_eq(0., DEFAULT_F64_MARGIN) {
                    return Ok(neighbor)
                }
                neighbors.push(Reverse(neighbor));
            }
        }
        neighbors.pop().map(|reverse| reverse.0).ok_or(String::from("no neighbors"))
    }
}


impl<S: Spatial> Eq for Neighbor<S> {}

impl<S: Spatial> Ord for Neighbor<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}

impl<S: Spatial> PartialEq for Neighbor<S> {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl<S: Spatial> PartialOrd for Neighbor<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

struct Shifts {
    e: Point,
    ne: Point,
    n: Point,
    nw: Point,
    w: Point,
    sw: Point,
    s: Point,
    se: Point,
}

impl Shifts {
    fn new(radius: f64) -> Shifts {
        Shifts {
            e:  E.mul(radius),
            ne: NE.mul(radius),
            n:  N.mul(radius),
            nw: NW.mul(radius),
            w:  W.mul(radius),
            sw: SW.mul(radius),
            s:  S.mul(radius),
            se: SE.mul(radius),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tile::*;

    #[test]
    pub fn test_tree() {
        let mut tree = Tree::new(Config {
            initial_radius: 100.,
            max_depth: 40,
            splitting_threshold: 10,
        });

        let square = ProtoTile::new(vec![Point(0., 0.), Point(1., 0.), Point(1., 1.), Point(0., 1.)]);

        tree.insert(Tile::new(square.transform(&Euclid::Translate((0., 0.))), false));
        tree.insert(Tile::new(square.transform(&Euclid::Translate((2., 2.))), false));
        tree.insert(Tile::new(square.transform(&Euclid::Translate((-4., -4.))), false));

        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
        tree.nearest_neighbor(&Point(0.5, 0.5)).expect("foo");
    }

    #[test]
    pub fn test_bounding_leaf() {
        let tree = Tree::<Point, Point> {
            config: Config {
                initial_radius: 100.0,
                max_depth: 40,
                splitting_threshold: 10,
            },
            root: Node {
                node: NodeType::InnerNode(Box::new(
                    InnerNode {
                        bounds: Bounds {
                            center: Point(
                                0.0,
                                0.0,
                            ),
                            radius: 100.0,
                        },
                        level: 0,
                        ne: Node {
                            node: NodeType::InnerNode(Box::new(
                                InnerNode {
                                    bounds: Bounds {
                                        center: Point(
                                            50.0,
                                            50.0,
                                        ),
                                        radius: 50.0,
                                    },
                                    level: 1,
                                    ne: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        75.0,
                                                        75.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    nw: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        25.0,
                                                        75.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    se: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        75.0,
                                                        25.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    sw: Node {
                                        node: NodeType::InnerNode(Box::new(
                                            InnerNode {
                                                bounds: Bounds {
                                                    center: Point(
                                                        25.0,
                                                        25.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                level: 2,
                                                ne: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    37.5,
                                                                    37.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                nw: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    12.5,
                                                                    37.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                se: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    37.5,
                                                                    12.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                sw: Node {
                                                    node: NodeType::InnerNode(Box::new(
                                                        InnerNode {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    12.5,
                                                                    12.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            level: 3,
                                                            ne: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                18.75,
                                                                                18.75,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            nw: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                6.25,
                                                                                18.75,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            se: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                18.75,
                                                                                6.25,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            sw: Node {
                                                                node: NodeType::InnerNode(Box::new(
                                                                    InnerNode {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                6.25,
                                                                                6.25,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        level: 4,
                                                                        ne: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            9.375,
                                                                                            9.375,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        nw: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            3.125,
                                                                                            9.375,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        se: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            9.375,
                                                                                            3.125,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        sw: Node {
                                                                            node: NodeType::InnerNode(Box::new(
                                                                                InnerNode {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            3.125,
                                                                                            3.125,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    level: 5,
                                                                                    ne: Node {
                                                                                        node: NodeType::InnerNode(Box::new(
                                                                                            InnerNode {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        4.6875,
                                                                                                        4.6875,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                level: 6,
                                                                                                ne: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    5.46875,
                                                                                                                    5.46875,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                nw: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    3.90625,
                                                                                                                    5.46875,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                se: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    5.46875,
                                                                                                                    3.90625,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                sw: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    3.90625,
                                                                                                                    3.90625,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                            },
                                                                                        )),
                                                                                    },
                                                                                    nw: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        1.5625,
                                                                                                        4.6875,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    se: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        4.6875,
                                                                                                        1.5625,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    sw: Node {
                                                                                        node: NodeType::InnerNode(Box::new(
                                                                                            InnerNode {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        1.5625,
                                                                                                        1.5625,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                level: 6,
                                                                                                ne: Node {
                                                                                                    node: NodeType::InnerNode(Box::new(
                                                                                                        InnerNode {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    2.34375,
                                                                                                                    2.34375,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            level: 7,
                                                                                                            ne: Node {
                                                                                                                node: NodeType::Leaf(
                                                                                                                    Leaf {
                                                                                                                        bounds: Bounds {
                                                                                                                            center: Point(
                                                                                                                                2.734375,
                                                                                                                                2.734375,
                                                                                                                            ),
                                                                                                                            radius: 0.390625,
                                                                                                                        },
                                                                                                                        items: vec![],
                                                                                                                        level: 8,
                                                                                                                    },
                                                                                                                ),
                                                                                                            },
                                                                                                            nw: Node {
                                                                                                                node: NodeType::Leaf(
                                                                                                                    Leaf {
                                                                                                                        bounds: Bounds {
                                                                                                                            center: Point(
                                                                                                                                1.953125,
                                                                                                                                2.734375,
                                                                                                                            ),
                                                                                                                            radius: 0.390625,
                                                                                                                        },
                                                                                                                        items: vec![],
                                                                                                                        level: 8,
                                                                                                                    },
                                                                                                                ),
                                                                                                            },
                                                                                                            se: Node {
                                                                                                                node: NodeType::Leaf(
                                                                                                                    Leaf {
                                                                                                                        bounds: Bounds {
                                                                                                                            center: Point(
                                                                                                                                2.734375,
                                                                                                                                1.953125,
                                                                                                                            ),
                                                                                                                            radius: 0.390625,
                                                                                                                        },
                                                                                                                        items: vec![],
                                                                                                                        level: 8,
                                                                                                                    },
                                                                                                                ),
                                                                                                            },
                                                                                                            sw: Node {
                                                                                                                node: NodeType::Leaf(
                                                                                                                    Leaf {
                                                                                                                        bounds: Bounds {
                                                                                                                            center: Point(
                                                                                                                                1.953125,
                                                                                                                                1.953125,
                                                                                                                            ),
                                                                                                                            radius: 0.390625,
                                                                                                                        },
                                                                                                                        items: vec![],
                                                                                                                        level: 8,
                                                                                                                    },
                                                                                                                ),
                                                                                                            },
                                                                                                        },
                                                                                                    )),
                                                                                                },
                                                                                                nw: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    0.78125,
                                                                                                                    2.34375,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                se: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    2.34375,
                                                                                                                    0.78125,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                sw: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    0.78125,
                                                                                                                    0.78125,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                            },
                                                                                        )),
                                                                                    },
                                                                                },
                                                                            )),
                                                                        },
                                                                    },
                                                                )),
                                                            },
                                                        },
                                                    )),
                                                },
                                            },
                                        )),
                                    },
                                },
                            )),
                        },
                        nw: Node {
                            node: NodeType::Leaf(
                                Leaf {
                                    bounds: Bounds {
                                        center: Point(
                                            -50.0,
                                            50.0,
                                        ),
                                        radius: 50.0,
                                    },
                                    items: vec![],
                                    level: 1,
                                },
                            ),
                        },
                        se: Node {
                            node: NodeType::InnerNode(Box::new(
                                InnerNode {
                                    bounds: Bounds {
                                        center: Point(
                                            50.0,
                                            -50.0,
                                        ),
                                        radius: 50.0,
                                    },
                                    level: 1,
                                    ne: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        75.0,
                                                        -25.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    nw: Node {
                                        node: NodeType::InnerNode(Box::new(
                                            InnerNode {
                                                bounds: Bounds {
                                                    center: Point(
                                                        25.0,
                                                        -25.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                level: 2,
                                                ne: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    37.5,
                                                                    -12.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                nw: Node {
                                                    node: NodeType::InnerNode(Box::new(
                                                        InnerNode {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    12.5,
                                                                    -12.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            level: 3,
                                                            ne: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                18.75,
                                                                                -6.25,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            nw: Node {
                                                                node: NodeType::InnerNode(Box::new(
                                                                    InnerNode {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                6.25,
                                                                                -6.25,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        level: 4,
                                                                        ne: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            9.375,
                                                                                            -3.125,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        nw: Node {
                                                                            node: NodeType::InnerNode(Box::new(
                                                                                InnerNode {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            3.125,
                                                                                            -3.125,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    level: 5,
                                                                                    ne: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        4.6875,
                                                                                                        -1.5625,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    nw: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        1.5625,
                                                                                                        -1.5625,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    se: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        4.6875,
                                                                                                        -4.6875,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    sw: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        1.5625,
                                                                                                        -4.6875,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                },
                                                                            )),
                                                                        },
                                                                        se: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            9.375,
                                                                                            -9.375,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        sw: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            3.125,
                                                                                            -9.375,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                    },
                                                                )),
                                                            },
                                                            se: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                18.75,
                                                                                -18.75,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            sw: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                6.25,
                                                                                -18.75,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                        },
                                                    )),
                                                },
                                                se: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    37.5,
                                                                    -37.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                sw: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    12.5,
                                                                    -37.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                            },
                                        )),
                                    },
                                    se: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        75.0,
                                                        -75.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    sw: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        25.0,
                                                        -75.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                },
                            )),
                        },
                        sw: Node {
                            node: NodeType::InnerNode(Box::new(
                                InnerNode {
                                    bounds: Bounds {
                                        center: Point(
                                            -50.0,
                                            -50.0,
                                        ),
                                        radius: 50.0,
                                    },
                                    level: 1,
                                    ne: Node {
                                        node: NodeType::InnerNode(Box::new(
                                            InnerNode {
                                                bounds: Bounds {
                                                    center: Point(
                                                        -25.0,
                                                        -25.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                level: 2,
                                                ne: Node {
                                                    node: NodeType::InnerNode(Box::new(
                                                        InnerNode {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    -12.5,
                                                                    -12.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            level: 3,
                                                            ne: Node {
                                                                node: NodeType::InnerNode(Box::new(
                                                                    InnerNode {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                -6.25,
                                                                                -6.25,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        level: 4,
                                                                        ne: Node {
                                                                            node: NodeType::InnerNode(Box::new(
                                                                                InnerNode {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            -3.125,
                                                                                            -3.125,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    level: 5,
                                                                                    ne: Node {
                                                                                        node: NodeType::InnerNode(Box::new(
                                                                                            InnerNode {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        -1.5625,
                                                                                                        -1.5625,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                level: 6,
                                                                                                ne: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    -0.78125,
                                                                                                                    -0.78125,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                nw: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    -2.34375,
                                                                                                                    -0.78125,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                se: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    -0.78125,
                                                                                                                    -2.34375,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                                sw: Node {
                                                                                                    node: NodeType::Leaf(
                                                                                                        Leaf {
                                                                                                            bounds: Bounds {
                                                                                                                center: Point(
                                                                                                                    -2.34375,
                                                                                                                    -2.34375,
                                                                                                                ),
                                                                                                                radius: 0.78125,
                                                                                                            },
                                                                                                            items: vec![],
                                                                                                            level: 7,
                                                                                                        },
                                                                                                    ),
                                                                                                },
                                                                                            },
                                                                                        )),
                                                                                    },
                                                                                    nw: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        -4.6875,
                                                                                                        -1.5625,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    se: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        -1.5625,
                                                                                                        -4.6875,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                    sw: Node {
                                                                                        node: NodeType::Leaf(
                                                                                            Leaf {
                                                                                                bounds: Bounds {
                                                                                                    center: Point(
                                                                                                        -4.6875,
                                                                                                        -4.6875,
                                                                                                    ),
                                                                                                    radius: 1.5625,
                                                                                                },
                                                                                                items: vec![],
                                                                                                level: 6,
                                                                                            },
                                                                                        ),
                                                                                    },
                                                                                },
                                                                            )),
                                                                        },
                                                                        nw: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            -9.375,
                                                                                            -3.125,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        se: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            -3.125,
                                                                                            -9.375,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                        sw: Node {
                                                                            node: NodeType::Leaf(
                                                                                Leaf {
                                                                                    bounds: Bounds {
                                                                                        center: Point(
                                                                                            -9.375,
                                                                                            -9.375,
                                                                                        ),
                                                                                        radius: 3.125,
                                                                                    },
                                                                                    items: vec![],
                                                                                    level: 5,
                                                                                },
                                                                            ),
                                                                        },
                                                                    },
                                                                )),
                                                            },
                                                            nw: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                -18.75,
                                                                                -6.25,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            se: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                -6.25,
                                                                                -18.75,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                            sw: Node {
                                                                node: NodeType::Leaf(
                                                                    Leaf {
                                                                        bounds: Bounds {
                                                                            center: Point(
                                                                                -18.75,
                                                                                -18.75,
                                                                            ),
                                                                            radius: 6.25,
                                                                        },
                                                                        items: vec![],
                                                                        level: 4,
                                                                    },
                                                                ),
                                                            },
                                                        },
                                                    )),
                                                },
                                                nw: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    -37.5,
                                                                    -12.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                se: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    -12.5,
                                                                    -37.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                                sw: Node {
                                                    node: NodeType::Leaf(
                                                        Leaf {
                                                            bounds: Bounds {
                                                                center: Point(
                                                                    -37.5,
                                                                    -37.5,
                                                                ),
                                                                radius: 12.5,
                                                            },
                                                            items: vec![],
                                                            level: 3,
                                                        },
                                                    ),
                                                },
                                            },
                                        )),
                                    },
                                    nw: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        -75.0,
                                                        -25.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    se: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        -25.0,
                                                        -75.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                    sw: Node {
                                        node: NodeType::Leaf(
                                            Leaf {
                                                bounds: Bounds {
                                                    center: Point(
                                                        -75.0,
                                                        -75.0,
                                                    ),
                                                    radius: 25.0,
                                                },
                                                items: vec![],
                                                level: 2,
                                            },
                                        ),
                                    },
                                },
                            )),
                        },
                    },
                )),
            },
            items: HashMap::new(),
        };

        tree.root.bounding_leaf(&Point(0.3666666666666686, 2.196666463216146)).ok_or("foo").expect("foo");
    }
}
