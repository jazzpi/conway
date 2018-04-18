//! The actual backend (i.e., the magic happens here).

pub mod data;

/// A 2D, integer point
pub type Point = (i32, i32);

pub fn point_minmax(a: Point, b: Point) -> (Point, Point) {
    if a.0 < b.0 {
        if a.1 < b.1 {
            ((a.0, a.1), (b.0, b.1))
        } else {
            ((a.0, b.1), (b.0, a.1))
        }
    } else {
        if a.1 < b.1 {
            ((b.0, a.1), (a.0, b.1))
        } else {
            ((b.0, b.1), (a.0, a.1))
        }
    }
}

#[cfg(test)]
#[allow(unused_results)]
mod tests {
    use super::*;

    mod functions {
        use super::{Point, point_minmax};

        fn minmax_check(points: (Point, Point)) {
            let (a, b) = points;
            assert_eq!(a, (-1, -1));
            assert_eq!(b, (2, 3));
        }

        #[test]
        fn minmax() {
            minmax_check(point_minmax((-1, -1), (2, 3)));
            minmax_check(point_minmax((-1, 3), (2, -1)));
            minmax_check(point_minmax((2, -1), (-1, 3)));
            minmax_check(point_minmax((2, 3), (-1, -1)));
        }
    }
}
