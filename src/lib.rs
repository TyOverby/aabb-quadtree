#![deny(missing_docs)]

//! A simple spacial partitioning data structure that allows fast queries for
//! 2-dimensional objects.
//!
//! As the name implies, the tree is a mapping from axis-aligned-bounding-box => object.

extern crate fnv;

pub mod geom;

use geom::{Point, Rect};
use fnv::FnvHasher;
use std::cmp::Ord;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

/// An object that has a bounding box.
///
/// Implementing this trait is not required, but can make insertions easier.
pub trait Spatial {
    /// Returns the boudning box for the object.
    fn aabb(&self) -> Rect;
}

/// An ID unique to a single QuadTree.  This is the object that is
/// returned from queries, and can be used to access the elements stored
/// in the quad tree.
///
/// DO NOT use an ItemId on a quadtree unless the ItemId came from that tree.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct ItemId(u32);

#[derive(Debug, Clone)]
struct QuadTreeConfig {
    allow_duplicates: bool,
    max_children: usize,
    min_children: usize,
    max_depth: usize,
    epsilon: f32,
}

/// The main QuadTree structure.  Mainly supports inserting, removing,
/// and querying objects in 3d space.
#[derive(Debug, Clone)]
pub struct QuadTree<T> {
    root: QuadNode,
    config: QuadTreeConfig,
    id: u32,
    elements: FnvHashMap<ItemId, (T, Rect)>,
}

#[derive(Debug)]
enum QuadNode {
    Branch {
        aabb: Rect,
        children: [(Rect, Box<QuadNode>); 4],
        in_all: Vec<(ItemId, Rect)>,
        element_count: usize,
        depth: usize,
    },
    Leaf {
        aabb: Rect,
        elements: Vec<(ItemId, Rect)>,
        depth: usize,
    },
}

impl Clone for QuadNode {
    fn clone(&self) -> QuadNode {
        match self {
            &QuadNode::Branch {
                ref aabb,
                ref children,
                ref in_all,
                ref element_count,
                ref depth,
            } => {
                let children = [
                    children[0].clone(),
                    children[1].clone(),
                    children[2].clone(),
                    children[3].clone(),
                ];
                QuadNode::Branch {
                    aabb: aabb.clone(),
                    children: children,
                    in_all: in_all.clone(),
                    element_count: element_count.clone(),
                    depth: depth.clone(),
                }
            }
            &QuadNode::Leaf {
                ref aabb,
                ref elements,
                ref depth,
            } => QuadNode::Leaf {
                aabb: aabb.clone(),
                elements: elements.clone(),
                depth: depth.clone(),
            },
        }
    }
}

impl<T> QuadTree<T> {
    /// Constructs a new QuadTree with customizable options.
    ///
    /// * `size`: the enclosing space for the quad-tree.
    /// * `allow_duplicates`: if false, the quadtree will remove objects that have the same bounding box.
    /// * `min_children`: the minimum amount of children that a tree node will have.
    /// * `max_children`: the maximum amount of children that a tree node will have before it gets split.
    /// * `max_depth`: the maximum depth that the tree can grow before it stops.
    pub fn new(size: Rect, allow_duplicates: bool, min_children: usize, max_children: usize, max_depth: usize) -> QuadTree<T> {
        QuadTree {
            root: QuadNode::Leaf {
                aabb: size,
                elements: Vec::with_capacity(max_children),
                depth: 0,
            },
            config: QuadTreeConfig {
                allow_duplicates: allow_duplicates,
                max_children: max_children,
                min_children: min_children,
                max_depth: max_depth,
                epsilon: 0.0001,
            },
            id: 0,
            elements: HashMap::with_capacity_and_hasher(max_children * 16, Default::default()),
        }
    }

    /// Constructs a new QuadTree with customizable options.
    ///
    /// * `size`: the enclosing space for the quad-tree.
    /// ### Defauts
    /// * `allow_duplicates`: true
    /// * `min_children`: 4
    /// * `max_children`: 16
    /// * `max_depth`: 8
    pub fn default(size: Rect) -> QuadTree<T> { QuadTree::new(size, true, 4, 16, 8) }

    /// Inserts an element with the provided bounding box.
    pub fn insert_with_box(&mut self, t: T, aabb: Rect) -> ItemId {
        let &mut QuadTree {
            ref mut root,
            ref config,
            ref mut id,
            ref mut elements,
        } = self;

        let item_id = ItemId(*id);
        *id += 1;

        if root.insert(item_id, aabb, config) {
            elements.insert(item_id, (t, aabb));
        }

        item_id
    }

    /// Returns an ItemId for the first element that was inserted into the tree.
    pub fn first(&self) -> Option<ItemId> { self.elements.iter().next().map(|(id, _)| *id) }

    /// Inserts an element into the tree.
    pub fn insert(&mut self, t: T) -> ItemId
    where
        T: Spatial,
    {
        let b = t.aabb();
        self.insert_with_box(t, b)
    }

    /// Retrieves an element by looking it up from the ItemId.
    pub fn get(&self, id: ItemId) -> Option<&T> {
        self.elements.get(&id).map(|&(ref a, _)| a)
    }

    /// Returns an iterator of (element, bounding-box, id) for each element
    /// whose bounding box intersects with `bounding_box`.
    pub fn query(&self, bounding_box: Rect) -> Vec<(&T, &Rect, ItemId)>
    where
        T: ::std::fmt::Debug,
    {
        let mut ids = vec![];
        self.root.query(bounding_box, &mut ids);
        ids.sort_by_key(|&(id, _)| id);
        ids.dedup();
        ids.iter()
            .map(|&(id, _)| {
                let &(ref t, ref rect) = match self.elements.get(&id) {
                    Some(e) => e,
                    None => {
                        panic!("looked for {:?}", id);
                    }
                };
                (t, rect, id)
            })
            .collect()
    }

    /// Attempts to remove the item with id `item_id` from the tree.  If that
    /// item was present, it returns a tuple of (element, bounding-box)
    pub fn remove(&mut self, item_id: ItemId) -> Option<(T, Rect)> {
        match self.elements.remove(&item_id) {
            Some((item, aabb)) => {
                self.root.remove(item_id, aabb, &self.config);
                Some((item, aabb))
            }
            None => None,
        }
    }

    /// Returns an iterator over all the items in the tree.
    pub fn iter(&self) -> ::std::collections::hash_map::Iter<ItemId, (T, Rect)> { self.elements.iter() }

    /// Calls `f` repeatedly for every node in the tree with these arguments
    ///
    /// * `&Rect`: The boudning box of that tree node
    /// * `usize`: The current depth
    /// * `bool`: True if the node is a leaf-node, False if the node is a branch node.
    pub fn inspect<F: FnMut(&Rect, usize, bool)>(&self, mut f: F) { self.root.inspect(&mut f); }

    /// Returns the number of elements in the tree
    pub fn len(&self) -> usize { self.elements.len() }

    /// Returns true if the tree is empty.
    pub fn is_empty(&self) -> bool { self.elements.is_empty() }

    /// Returns the enclosing bounding-box for the entire tree.
    pub fn bounding_box(&self) -> Rect {
        self.root.bounding_box()
    }
}

impl QuadNode {
    fn bounding_box(&self) -> Rect {
        match self {
            &QuadNode::Branch { ref aabb, .. } => aabb.clone(),
            &QuadNode::Leaf { ref aabb, .. } => aabb.clone(),
        }
    }

    fn new_leaf(aabb: Rect, depth: usize, config: &QuadTreeConfig) -> QuadNode {
        QuadNode::Leaf {
            aabb: aabb,
            elements: Vec::with_capacity(config.max_children / 2),
            depth: depth,
        }
    }

    fn inspect<F: FnMut(&Rect, usize, bool)>(&self, f: &mut F) {
        match self {
            &QuadNode::Branch {
                depth,
                ref aabb,
                ref children,
                ..
            } => {
                f(aabb, depth, false);
                for child in children {
                    child.1.inspect(f);
                }
            }
            &QuadNode::Leaf { depth, ref aabb, .. } => {
                f(aabb, depth, true);
            }
        }
    }

    fn insert(&mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig) -> bool {
        let mut into = None;
        let mut did_insert = false;
        match self {
            &mut QuadNode::Branch {
                ref aabb,
                ref mut in_all,
                ref mut children,
                ref mut element_count,
                ..
            } => {
                if item_aabb.contains(&aabb.midpoint()) {
                    // Only insert if there isn't another item with a very
                    // similar aabb.
                    if config.allow_duplicates || !in_all.iter().any(|&(_, ref e_bb)| e_bb.close_to(&item_aabb, config.epsilon)) {
                        in_all.push((item_id, item_aabb));
                        did_insert = true;
                        *element_count += 1;
                    }
                } else {
                    for &mut (ref aabb, ref mut child) in children {
                        if aabb.does_intersect(&item_aabb) {
                            if child.insert(item_id, item_aabb, config) {
                                *element_count += 1;
                                did_insert = true;
                            }
                        }
                    }
                }
            }

            &mut QuadNode::Leaf {
                ref aabb,
                ref mut elements,
                ref depth,
            } => {
                if elements.len() == config.max_children && *depth != config.max_depth {
                    // STEAL ALL THE CHILDREN MUAHAHAHAHA
                    let mut extracted_children = Vec::new();
                    ::std::mem::swap(&mut extracted_children, elements);
                    extracted_children.push((item_id, item_aabb));
                    did_insert = true;

                    let split = aabb.split_quad();
                    into = Some((
                        extracted_children,
                        QuadNode::Branch {
                            aabb: *aabb,
                            in_all: Vec::new(),
                            children: [
                                (split[0], Box::new(QuadNode::new_leaf(split[0], depth + 1, config))),
                                (split[1], Box::new(QuadNode::new_leaf(split[1], depth + 1, config))),
                                (split[2], Box::new(QuadNode::new_leaf(split[2], depth + 1, config))),
                                (split[3], Box::new(QuadNode::new_leaf(split[3], depth + 1, config))),
                            ],
                            element_count: 0,
                            depth: *depth,
                        },
                    ));
                } else {
                    if config.allow_duplicates ||
                        !elements
                            .iter()
                            .any(|&(_, ref e_bb)| e_bb.close_to(&item_aabb, config.epsilon))
                    {
                        elements.push((item_id, item_aabb));
                        did_insert = true;
                    }
                }
            }
        }

        // If we transitioned from a leaf node to a branch node, we
        // need to update ourself and re-add all the children that
        // we used to have
        // in our this leaf into our new leaves.
        if let Some((extracted_children, new_node)) = into {
            *self = new_node;
            for (child_id, child_aabb) in extracted_children {
                self.insert(child_id, child_aabb, config);
            }
        }

        did_insert
    }

    fn remove(&mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig) -> bool {
        fn remove_from(v: &mut Vec<(ItemId, Rect)>, item: ItemId) -> bool {
            if let Some(index) = v.iter().position(|a| a.0 == item) {
                v.swap_remove(index);
                true
            } else {
                false
            }
        }

        let mut compact = None;
        let removed = match self {
            &mut QuadNode::Branch {
                ref depth,
                ref aabb,
                ref mut in_all,
                ref mut children,
                ref mut element_count,
                ..
            } => {
                let mut did_remove = false;

                if item_aabb.contains(&aabb.midpoint()) {
                    did_remove = remove_from(in_all, item_id);
                } else {
                    for &mut (ref child_aabb, ref mut child_tree) in children {
                        if child_aabb.does_intersect(&item_aabb) {
                            did_remove |= child_tree.remove(item_id, item_aabb, config);
                        }
                    }
                }

                if did_remove {
                    *element_count -= 1;
                    if *element_count < config.min_children {
                        compact = Some((*element_count, *aabb, *depth));
                    }
                }
                did_remove
            }

            &mut QuadNode::Leaf { ref mut elements, .. } => remove_from(elements, item_id),
        };

        if let Some((size, aabb, depth)) = compact {
            let mut elements = Vec::with_capacity(size);
            self.query(aabb, &mut elements);
            elements.sort_by(|&(id1, _), &(ref id2, _)| id1.cmp(id2));
            elements.dedup();
            *self = QuadNode::Leaf {
                aabb: aabb,
                elements: elements,
                depth: depth,
            };
        }
        removed
    }

    fn query(&self, query_aabb: Rect, out: &mut Vec<(ItemId, Rect)>) {
        fn match_all(elements: &Vec<(ItemId, Rect)>, query_aabb: Rect, out: &mut Vec<(ItemId, Rect)>) {
            for &(ref child_id, ref child_aabb) in elements {
                if query_aabb.does_intersect(child_aabb) {
                    out.push((*child_id, *child_aabb))
                }
            }
        }

        match self {
            &QuadNode::Branch { ref in_all, ref children, .. } => {
                match_all(in_all, query_aabb, out);

                for &(ref child_aabb, ref child_tree) in children {
                    if query_aabb.does_intersect(&child_aabb) {
                        child_tree.query(query_aabb, out);
                    }
                }
            }
            &QuadNode::Leaf { ref elements, .. } => match_all(elements, query_aabb, out),
        }
    }
}

impl Spatial for Rect {
    fn aabb(&self) -> Rect { *self }
}

impl Spatial for Point {
    fn aabb(&self) -> Rect { Rect::null_at(self) }
}

#[test]
fn similar_points() {
    let mut quad_tree = QuadTree::new(Rect::centered_with_radius(&Point { x: 0.0, y: 0.0 }, 10.0), false, 1, 5, 2);

    let p = Point { x: 0.0, y: 0.0 };
    quad_tree.insert(p);
    quad_tree.insert(p);
    assert_eq!(quad_tree.elements.len(), 1);
}
