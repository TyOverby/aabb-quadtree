#![allow(dead_code, missing_docs)]


use std::ops::{Add, Neg, Sub};

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct Rect {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector { Vector { x: -self.x, y: -self.y } }
}

impl Sub<Vector> for Point {
    type Output = Point;
    fn sub(self, rhs: Vector) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, rhs: Vector) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Point> for Vector {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add<Point> for Vector {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}


impl Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, rhs: Point) -> Vector {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Rect {
    pub fn centered_with_radius(p1: &Point, radius: f32) -> Rect {
        let v = Vector { x: radius, y: radius };
        Rect::from_points(&(*p1 - v), &(*p1 + v))
    }

    pub fn from_points(p1: &Point, p2: &Point) -> Rect {
        let mut r = Rect::null_at(&p1);
        r.expand_to_include(&p2);
        r
    }

    pub fn from_point_and_size(point: &Point, size: &Vector) -> Rect {
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
        Rect {
            top_left: *point,
            bottom_right: *point + *size,
        }
    }

    pub fn null() -> Rect {
        let nan = ::std::f32::NAN;
        Rect {
            top_left: Point { x: nan, y: nan },
            bottom_right: Point { x: nan, y: nan },
        }
    }

    pub fn null_at(point: &Point) -> Rect {
        Rect {
            top_left: *point,
            bottom_right: *point,
        }
    }

    pub fn expand(&self, left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        let top_left_vec = Vector { x: left, y: top };
        let bottom_right_vec = Vector { x: right, y: bottom };
        Rect {
            top_left: self.top_left - top_left_vec,
            bottom_right: self.bottom_right + bottom_right_vec,
        }
    }

    pub fn width(&self) -> f32 { self.bottom_right.x - self.top_left.x }

    pub fn height(&self) -> f32 { self.bottom_right.y - self.top_left.y }

    pub fn left(&self) -> f32 { self.top_left.x }

    pub fn right(&self) -> f32 { self.bottom_right.x }

    pub fn top(&self) -> f32 { self.top_left.y }

    pub fn bottom(&self) -> f32 { self.bottom_right.y }

    pub fn top_left(&self) -> Point { self.top_left }

    pub fn bottom_right(&self) -> Point { self.bottom_right }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.top_left().x,
            y: self.bottom_right().y,
        }
    }

    pub fn top_right(&self) -> Point {
        Point {
            x: self.bottom_right().x,
            y: self.top_left().y,
        }
    }

    pub fn north(&self) -> Point {
        Point {
            x: self.left() + self.width() / 2.0,
            y: self.top(),
        }
    }

    pub fn south(&self) -> Point {
        Point {
            x: self.left() + self.width() / 2.0,
            y: self.bottom(),
        }
    }

    pub fn west(&self) -> Point {
        Point {
            x: self.left(),
            y: self.top() + self.height() / 2.0,
        }
    }

    pub fn east(&self) -> Point {
        Point {
            x: self.right(),
            y: self.top() + self.height() / 2.0,
        }
    }


    pub fn expanded_by(&self, point: &Point) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(point);
        r
    }

    pub fn is_null(&self) -> bool {
        self.top_left.x.is_nan() || self.top_left.y.is_nan() || self.bottom_right.x.is_nan() || self.bottom_right.y.is_nan()
    }

    pub fn expand_to_include(&mut self, point: &Point) {
        fn min(a: f32, b: f32) -> f32 {
            if a.is_nan() { return b; }
            if b.is_nan() { return a; }
            if a < b { return a; }
            return b;
        }

        fn max(a: f32, b: f32) -> f32 {
            if a.is_nan() { return b; }
            if b.is_nan() { return a; }
            if a > b { return a; }
            return b;
        }

        self.top_left.x = min(self.top_left.x, point.x);
        self.top_left.y = min(self.top_left.y, point.y);

        self.bottom_right.x = max(self.bottom_right.x, point.x);
        self.bottom_right.y = max(self.bottom_right.y, point.y);
    }

    pub fn union_with(&self, other: &Rect) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(&other.top_left);
        r.expand_to_include(&other.bottom_right);
        r
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.top_left.x && p.x < self.bottom_right.x && p.y >= self.top_left.y && p.y < self.bottom_right.y
    }

    pub fn does_intersect(&self, other: &Rect) -> bool {
        let r1 = self;
        let r2 = other;

        // From stack overflow:
        // http://gamedev.stackexchange.com/a/913
        !(r2.left() > r1.right() || r2.right() < r1.left() || r2.top() > r1.bottom() || r2.bottom() < r1.top())
    }

    pub fn intersect_with(&self, other: &Rect) -> Rect {
        if !self.does_intersect(other) {
            return Rect::null();
        }
        let left = self.left().max(other.left());
        let right = self.right().min(other.right());

        let top = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());

        Rect::from_points(&Point { x: left, y: top }, &Point { x: right, y: bottom })
    }

    pub fn midpoint(&self) -> Point {
        let half = Vector {
            x: self.width() / 2.0,
            y: self.height() / 2.0,
        };
        self.top_left() + half
    }

    pub fn split_vert(&self) -> (Rect, Rect) {
        let half_size = Vector {
            x: self.width() / 2.0,
            y: self.height(),
        };
        let half_offset = Vector { x: self.width() / 2.0, y: 0.0 };
        (
            Rect::from_point_and_size(&self.top_left, &half_size),
            Rect::from_point_and_size(&(self.top_left + half_offset), &half_size),
        )
    }

    pub fn split_hori(&self) -> (Rect, Rect) {
        let half_size = Vector {
            x: self.width(),
            y: self.height() / 2.0,
        };
        let half_offset = Vector { x: 0.0, y: self.height() / 2.0 };
        (
            Rect::from_point_and_size(&self.top_left, &half_size),
            Rect::from_point_and_size(&(self.top_left + half_offset), &half_size),
        )
    }

    pub fn split_quad(&self) -> [Rect; 4] {
        let half = Vector {
            x: self.width() / 2.0,
            y: self.height() / 2.0,
        };
        [
            // x _
            // _ _
            Rect::from_point_and_size(&self.top_left, &half),
            // _ x
            // _ _
            Rect::from_point_and_size(
                &Point {
                    x: self.top_left.x + half.x,
                    ..self.top_left
                },
                &half,
            ),
            // _ _
            // x _
            Rect::from_point_and_size(
                &Point {
                    y: self.top_left.y + half.y,
                    ..self.top_left
                },
                &half,
            ),
            // _ _
            // _ x
            Rect::from_point_and_size(&(self.top_left + half), &half),
        ]
    }

    pub fn close_to(&self, other: &Rect, epsilon: f32) -> bool {
        self.top_left.close_to(&other.top_left, epsilon) && self.bottom_right.close_to(&other.bottom_right, epsilon)
    }
}

impl Vector {
    pub fn magnitude(&self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }

    pub fn normalized(&self) -> Vector {
        let m = self.magnitude();
        Vector { x: self.x / m, y: self.y / m }
    }

    pub fn mul_e(&self, other: &Vector) -> Vector {
        Vector {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    pub fn scale_e(&self, sx: f32, sy: f32) -> Vector { Vector { x: self.x * sx, y: self.y * sy } }

    pub fn cross(&self, other: &Vector) -> f32 { self.x * other.y - self.y * other.x }

    pub fn dot(&self, other: &Vector) -> f32 { self.x * other.x + self.y * other.y }
}

impl Point {
    pub fn close_to(&self, other: &Point, epsilon: f32) -> bool { self.distance_2(other) < epsilon * epsilon }

    pub fn distance(&self, other: &Point) -> f32 { self.distance_2(other).sqrt() }

    pub fn distance_2(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}
