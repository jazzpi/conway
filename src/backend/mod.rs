//! The actual backend (i.e., the magic happens here).

type Point = (i32, i32);

/// Axis-aligned Bounding Box
///
/// The bottom and left borders are considered to be part of the bounding box,
/// the top and right borders are not.
#[derive(Debug, Clone)]
pub struct AABB {
    center: Point,
    half_dim: i32,
}

impl AABB {
    /// Create a new bounding box with a given `center` and half-dimension.
    pub fn new(center: Point, half_dim: i32) -> AABB {
        AABB {
            center,
            half_dim,
        }
    }

    /// Check if a given `point` is in the bounding box.
    pub fn contains(&self, point: Point) -> bool {
        let centered = (point.0 - self.center.0, point.1 - self.center.1);
        (-self.half_dim <= centered.0 && centered.0 < self.half_dim &&
         -self.half_dim <= centered.1 && centered.1 < self.half_dim)
    }

    /// Check if the bounding box intersects another one.
    pub fn intersects(&self, other: &AABB) -> bool {
        (Self::intersects_range(self.x_range(), other.x_range()) &&
         Self::intersects_range(self.y_range(), other.y_range()))
    }

    fn intersects_range(a: (i32, i32), b: (i32, i32)) -> bool {
        (a.0 <= b.0 && a.1 > b.0 ||
         b.0 <= a.0 && b.1 > a.0)
    }

    fn x_range(&self) -> (i32, i32) {
        (self.center.0 - self.half_dim, self.center.0 + self.half_dim)
    }

    fn y_range(&self) -> (i32, i32) {
        (self.center.1 - self.half_dim, self.center.1 + self.half_dim)
    }
}

const QTREE_CAP: usize = 4;

/// A quadtree implementation with integer coordinates.
///
/// Automatically "reunites" sub-trees if possible and also automatically
/// extends its boundaries to fit new elements.
#[derive(Debug)]
pub struct QTree {
    boundary: AABB,
    points: Option<[Option<Point>; 4]>,
    children: Option<[Box<QTree>; 4]>,
}

impl QTree {
    /// Create a new quadtree with a given `boundary` and some initial points.
    pub fn new(boundary: AABB, elements: &Vec<Point>) -> QTree {
        let mut tree = QTree {
            boundary,
            points: Some([None, None, None, None]),
            children: None,
        };

        for point in elements {
            tree.set(*point);
        }

        tree
    }

    /// Check if there is something at a `point` in the quadtree.
    pub fn get(&self, point: Point) -> bool {
        if let Some(ref points) = self.points {
            points.contains(&Some(point))
        } else if let Some(ref children) = self.children {
            children[Self::get_child(&self.boundary, point)].get(point)
        } else {
            Self::invalid_state()
        }
    }

    /// Add a `point` to the quadtree.
    ///
    /// If the point is already set, do nothing.
    pub fn set(&mut self, point: Point) {
        if !self.boundary.contains(point) {
            let max_dist = f64::max(point.0.into(), point.1.into());
            let half_dim = max_dist.log2().ceil().exp2() as i32;
            let new_boundary = AABB::new(self.boundary.center, half_dim);
            self.extend(new_boundary);
        }

        let mut should_subdivide = false;
        if let Some(ref mut points) = self.points {
            for p in points {
                match p {
                    &mut Some(p) => {
                        if p == point {
                            // point already exists
                            return
                        }
                    }
                    &mut None => {
                        // Set point here
                        *p = Some(point);
                        return
                    }
                }
            }

            // Can't call subdivide here because we'd borrow mutably twice
            should_subdivide = true;
        }

        if should_subdivide {
            self.subdivide();
        }

        if let Some(ref mut children) = self.children {
            let child = Self::get_child(&self.boundary, point);
            children[child].set(point);
        } else {
            Self::invalid_state()
        }
    }

    /// Remove a `point` from the quadtree.
    ///
    /// If the point is not in the quadtree, do nothing.
    pub fn remove(&mut self, point: Point) {
        let mut check_union = false;

        if let Some(ref mut points) = self.points {
            let index = points.into_iter().position(|p| {
                *p == Some(point)
            });
            if let Some(index) = index {
                points[index] = None
            }
        } else if let Some(ref mut children) = self.children {
            let child = Self::get_child(&self.boundary, point);
            children[child].remove(point);

            check_union = true;
        } else {
            Self::invalid_state()
        }

        if check_union {
            self.check_union();
        }
    }

    /// Get a vector of all points in an area.
    pub fn query(&self, area: &AABB) -> Vec<Point> {
        if self.boundary.intersects(area) {
            if let Some(points) = self.points {
                points.iter().filter_map(|p| {
                    p.and_then(|p| {
                        if area.contains(p) {
                            Some(p)
                        } else {
                            None
                        }
                    })
                }).collect()
            } else if let Some(ref children) = self.children {
                children.iter().flat_map(|c| c.query(area)).collect()
            } else {
                Self::invalid_state()
            }
        } else {
            vec![]
        }
    }

    fn get_child(boundary: &AABB, point: Point) -> usize {
        if point.1 >= boundary.center.1 {
            if point.0 >= boundary.center.0 {
                0 // north-east
            } else {
                1 // north-west
            }
        } else {
            if point.0 >= boundary.center.0 {
                2 // south-east
            } else {
                3 // south-west
            }
        }
    }

    fn extend(&mut self, new_boundary: AABB) {
        self.boundary = new_boundary;

        // Collect these here because we can't borrow self (to call into_iter)
        // later
        let points: Vec<Point> = if self.children.is_some() {
            self.into_iter().collect()
        } else {
            vec![]
        };
        if let Some(ref mut children) = self.children {
            let bbs = Self::new_bbs(&self.boundary);
            for i in 0..4 {
                children[i] = Box::new(QTree::new(bbs[i].clone(), &vec![]));
            }
        }
        // Set these here because we can't borrow self (to call set) earlier
        if self.children.is_some() {
            for p in points {
                self.set(p);
            }
        }
    }

    /// Subdivide the quadtree into four children.
    ///
    /// Note: This only works properly if the half-dimension of our boundary is
    /// even.
    ///
    /// # Panics
    /// Panics if `children` is `None`
    fn subdivide(&mut self) {
        let bbs = Self::new_bbs(&self.boundary);
        let mut points = [vec![], vec![], vec![], vec![]];
        for point in self.points.unwrap().iter() {
            if let &Some(p) = point {
                for (i, bb) in bbs.iter().enumerate() {
                    if bb.contains(p) {
                        points[i].push(p);
                    }
                }
            }
        }

        self.points = None;
        self.children = Some([
            Box::new(QTree::new(bbs[0].clone(), &points[0])),
            Box::new(QTree::new(bbs[1].clone(), &points[1])),
            Box::new(QTree::new(bbs[2].clone(), &points[2])),
            Box::new(QTree::new(bbs[3].clone(), &points[3])),
        ]);
    }

    fn check_union(&mut self) {
        let mut points = [None; 4];
        {
            let it = self.into_iter().enumerate();
            for (i, p) in it {
                if i >= QTREE_CAP {
                    // Too many points remaining
                    return
                }
                points[i] = Some(p);
            }
        }

        self.points = Some(points);
        self.children = None;
    }

    fn new_bbs(old: &AABB) -> [AABB; 4] {
        let half_dim = old.half_dim / 2;
        let east = old.center.0 + half_dim;
        let west = old.center.0 - half_dim;
        let north = old.center.1 + half_dim;
        let south = old.center.1 - half_dim;
        [
            AABB::new((east, north), half_dim),
            AABB::new((west, north), half_dim),
            AABB::new((east, south), half_dim),
            AABB::new((west, south), half_dim),
        ]
    }

    fn invalid_state() -> ! {
        panic!("Invalid state - No children and no points!")
    }
}

/// An iterator over a `QTree`
pub struct QTreeIter<'a> {
    tree: &'a QTree,
    index: usize,
    child_iterator: Option<Box<QTreeIter<'a>>>,
}

impl<'a> Iterator for QTreeIter<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if let Some(ref mut it) = self.child_iterator {
            let n = it.next();
            if n.is_some() {
                return n
            } else {
                self.index += 1;
            }
        }
        if self.index == 4 {
            None
        } else if let Some(points) = self.tree.points {
            for p in &points[self.index..] {
                self.index += 1;
                if p.is_some() {
                    return *p
                }
            }
            None
        } else if let Some(ref children) = self.tree.children {
            while self.index < 4 {
                self.child_iterator = Some(Box::new(children[self.index].into_iter()));
                // Can't use unwrap because that moves the value out of the Option
                if let Some(ref mut it) = self.child_iterator {
                    let n = it.next();
                    if n.is_some() {
                        return n
                    } else {
                        self.index += 1;
                    }
                } else {
                    panic!("Some we just created became a None ?!?")
                }
            }
            None
        } else {
            QTree::invalid_state()
        }
    }
}

impl<'a> IntoIterator for &'a QTree {
    type Item = Point;
    type IntoIter = QTreeIter<'a>;

    fn into_iter(self) -> QTreeIter<'a> {
        QTreeIter {
            tree: self,
            index: 0,
            child_iterator: None,
        }
    }
}

#[cfg(test)]
#[allow(unused_results)]
mod tests {
    use super::*;

    mod aabb {
        use super::AABB;

        #[test]
        fn new() {
            AABB::new((10, 2), 4);
        }

        #[test]
        fn contains() {
            let bb = AABB::new((10, 2), 4);
            assert!(bb.contains((6, 3)));
            assert!(bb.contains((6, -2)));
            assert!(!bb.contains((10, 6)));
            assert!(!bb.contains((14, 6)));
            assert!(!bb.contains((6, -3)));
        }

        #[test]
        fn intersects() {
            let bb0 = AABB::new((10, 2), 4);
            let bb1 = AABB::new((6, 3), 1);
            let bb2 = AABB::new((6, -3), 1);
            let bb3 = AABB::new((11, 0), 1);
            assert!(bb0.intersects(&bb1));
            assert!(!bb0.intersects(&bb2));
            assert!(bb0.intersects(&bb3));
            assert!(bb3.intersects(&bb0));
        }
    }

    mod qtree {
        use super::{AABB, QTree, Point};
        use std::collections::BTreeSet;

        #[test]
        fn get_simple() {
            let tree = QTree::new(AABB::new((1, 3), 5),
                                  &vec![(0, 0), (4, 2), (0, -3)]);
            assert!(tree.get((0, 0)));
            assert!(tree.get((4, 2)));
            assert!(tree.get((0, -3)));
            assert!(!tree.get((-3, 0)));
        }

        #[test]
        fn set_simple() {
            let mut tree = QTree::new(AABB::new((1, 3), 5),
                                      &vec![(0, 0), (4, 2), (0, -3)]);
            let point = (-4, 2);
            tree.set(point);
            assert!(tree.get(point));
            tree.remove(point);
            assert!(!tree.get(point));
        }

        #[test]
        fn iterator_simple() {
            let mut tree = QTree::new(AABB::new((1, 3), 5),
                                  &vec![(0, 0), (4, 2)]);
            tree.set((0, 1));
            tree.set((0, -3));
            tree.remove((0, 1));
            let actual: BTreeSet<Point> = tree.into_iter().collect();
            let mut expected = BTreeSet::<Point>::new();
            expected.insert((0, 0));
            expected.insert((4, 2));
            expected.insert((0, -3));
            assert_eq!(actual, expected);
        }

        fn tree_with_children() -> QTree {
            let c1 = Box::new(QTree::new(AABB::new((2,   2), 2),
                                         &vec![(0, 0)]));
            let c2 = Box::new(QTree::new(AABB::new((-2,  2), 2),
                                         &vec![(-4, 0), (-1, 0)]));
            let c3 = Box::new(QTree::new(AABB::new((2,  -2), 2),
                                         &vec![(0, -4)]));
            let c4 = Box::new(QTree::new(AABB::new((-2, -2), 2),
                                         &vec![(-4, -4)]));
            QTree {
                boundary: AABB::new((0, 0), 4),
                points: None,
                children: Some([c1, c2, c3, c4]),
            }
        }

        #[test]
        fn get_children() {
            let tree = tree_with_children();

            assert!(tree.get((0, 0)));
            assert!(tree.get((-4, 0)));
            assert!(tree.get((0, -4)));
            assert!(tree.get((-4, -4)));
            assert!(tree.get((-1, 0)));
            assert!(!tree.get((0, 1)));
        }

        #[test]
        fn set_children() {
            let mut tree = tree_with_children();

            tree.set((0, -1));
            assert!(tree.get((0, -1)));
            assert!(tree.get((0, 0)));
            assert!(!tree.get((0, 1)));
            tree.remove((0, -1));
            tree.remove((0, 0));
            assert!(!tree.get((0, -1)));
            assert!(!tree.get((0, 0)));
        }

        #[test]
        fn iterator_children() {
            let mut tree = tree_with_children();

            tree.set((0, 1));
            tree.set((0, -3));
            tree.remove((0, 1));
            let actual: BTreeSet<Point> = tree.into_iter().collect();
            let mut expected = BTreeSet::<Point>::new();
            expected.insert((0, 0));
            expected.insert((-4, 0));
            expected.insert((0, -4));
            expected.insert((-4, -4));
            expected.insert((-1, 0));
            expected.insert((0, -3));
            assert_eq!(actual, expected);
        }

        #[test]
        fn set_divide() {
            let points = [
                (0, 0), (3, 2), (0, -3), (0, -4),
                (2, 2),
            ];
            let mut expected = BTreeSet::<Point>::new();
            let mut tree = QTree::new(AABB::new((0, 0), 4), &vec![]);
            for point in points.iter() {
                tree.set(*point);
                expected.insert(*point);
            }

            tree.set((2, 2));
            let actual: BTreeSet<Point> = tree.into_iter().collect();
            assert_eq!(actual, expected);
            assert!(tree.children.is_some());
            assert!(tree.points.is_none());

            tree.remove((2, 2));
            expected.remove(&(2, 2));
            let actual: BTreeSet<Point> = tree.into_iter().collect();
            assert_eq!(actual, expected);
            assert!(tree.children.is_none());
            assert!(tree.points.is_some());
        }

        #[test]
        fn nested() {
            let points = [
                (0, 0), (0, 1), (3, 2), (1, 3), (2, 0),
                (-4, 1), (-1, 3),
                (0, -3), (2, -1),
                (-1, -4),
            ];

            let mut expected = BTreeSet::<Point>::new();
            let mut tree = QTree::new(AABB::new((0, 0), 4), &vec![]);
            for point in points.iter() {
                tree.set(*point);
                expected.insert(*point);
            }

            let actual: BTreeSet<Point> = tree.into_iter().collect();
            assert_eq!(actual, expected);
        }

        #[test]
        fn extend() {
            let points = [
                (0, 0), (0, 1), (3, 2), (1, 3), (2, 0),
                (-4, 1), (-1, 3),
                (0, -3), (2, -1),
                (-1, -4),

                (9, 6),
            ];

            let mut expected = BTreeSet::<Point>::new();
            let mut tree = QTree::new(AABB::new((0, 0), 4), &vec![]);
            for point in points.iter() {
                tree.set(*point);
                expected.insert(*point);
            }

            let actual: BTreeSet<Point> = tree.into_iter().collect();
            assert_eq!(actual, expected);
            assert_eq!(tree.boundary.half_dim, 16);
        }
    }
}
