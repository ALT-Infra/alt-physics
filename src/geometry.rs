use crate::{Point, Side, Size};

pub(crate) const EPS: f64 = 1e-9;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Rect {
    pub center: Point,
    pub size: Size,
}

impl Rect {
    pub(crate) fn expanded(self, by: f64) -> Self {
        Self {
            center: self.center,
            size: Size::new(self.size.width + 2.0 * by, self.size.height + 2.0 * by),
        }
    }

    pub(crate) fn left(self) -> f64 {
        self.center.x - self.size.width * 0.5
    }
    pub(crate) fn right(self) -> f64 {
        self.center.x + self.size.width * 0.5
    }
    pub(crate) fn top(self) -> f64 {
        self.center.y - self.size.height * 0.5
    }
    pub(crate) fn bottom(self) -> f64 {
        self.center.y + self.size.height * 0.5
    }

    pub(crate) fn corners(self) -> [Point; 4] {
        [
            Point::new(self.left(), self.top()),
            Point::new(self.right(), self.top()),
            Point::new(self.right(), self.bottom()),
            Point::new(self.left(), self.bottom()),
        ]
    }

    pub(crate) fn overlaps(self, other: Self, clearance: f64) -> bool {
        (self.center.x - other.center.x).abs()
            < (self.size.width + other.size.width) * 0.5 + clearance
            && (self.center.y - other.center.y).abs()
                < (self.size.height + other.size.height) * 0.5 + clearance
    }

    pub(crate) fn contains_strict(self, p: Point) -> bool {
        p.x > self.left() + EPS
            && p.x < self.right() - EPS
            && p.y > self.top() + EPS
            && p.y < self.bottom() - EPS
    }
}

pub(crate) fn boundary_point(
    rect: Rect,
    toward: Point,
    side: Option<(Side, Option<f64>)>,
) -> Point {
    match side {
        Some((Side::Top, offset)) => Point::new(
            rect.center.x
                + offset
                    .unwrap_or_else(|| normalized_offset(rect.center.x, toward.x, rect.size.width))
                    * rect.size.width
                    * 0.5,
            rect.top(),
        ),
        Some((Side::Bottom, offset)) => Point::new(
            rect.center.x
                + offset
                    .unwrap_or_else(|| normalized_offset(rect.center.x, toward.x, rect.size.width))
                    * rect.size.width
                    * 0.5,
            rect.bottom(),
        ),
        Some((Side::Left, offset)) => Point::new(
            rect.left(),
            rect.center.y
                + offset.unwrap_or_else(|| {
                    normalized_offset(rect.center.y, toward.y, rect.size.height)
                }) * rect.size.height
                    * 0.5,
        ),
        Some((Side::Right, offset)) => Point::new(
            rect.right(),
            rect.center.y
                + offset.unwrap_or_else(|| {
                    normalized_offset(rect.center.y, toward.y, rect.size.height)
                }) * rect.size.height
                    * 0.5,
        ),
        None => {
            let dx = toward.x - rect.center.x;
            let dy = toward.y - rect.center.y;
            if dx.abs() < EPS && dy.abs() < EPS {
                return Point::new(rect.center.x, rect.bottom());
            }
            let tx = if dx.abs() < EPS {
                f64::INFINITY
            } else {
                rect.size.width * 0.5 / dx.abs()
            };
            let ty = if dy.abs() < EPS {
                f64::INFINITY
            } else {
                rect.size.height * 0.5 / dy.abs()
            };
            let t = tx.min(ty);
            Point::new(rect.center.x + dx * t, rect.center.y + dy * t)
        }
    }
}

fn normalized_offset(origin: f64, target: f64, span: f64) -> f64 {
    if span <= EPS {
        0.0
    } else {
        ((target - origin) / (span * 0.5)).clamp(-1.0, 1.0)
    }
}

pub(crate) fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> Option<(Point, f64)> {
    let r = Point::new(b.x - a.x, b.y - a.y);
    let s = Point::new(d.x - c.x, d.y - c.y);
    let denom = cross(r, s);
    if denom.abs() < EPS {
        return None;
    }
    let ca = Point::new(c.x - a.x, c.y - a.y);
    let t = cross(ca, s) / denom;
    let u = cross(ca, r) / denom;
    if t > EPS && t < 1.0 - EPS && u > EPS && u < 1.0 - EPS {
        let p = Point::new(a.x + t * r.x, a.y + t * r.y);
        let angle = crossing_angle(r, s);
        Some((p, angle))
    } else {
        None
    }
}

pub(crate) fn segment_hits_rect(a: Point, b: Point, rect: Rect) -> bool {
    if rect.contains_strict(a) || rect.contains_strict(b) {
        return true;
    }
    let corners = rect.corners();
    (0..4).any(|i| segments_intersect_inclusive(a, b, corners[i], corners[(i + 1) % 4]))
}

fn segments_intersect_inclusive(a: Point, b: Point, c: Point, d: Point) -> bool {
    let r = Point::new(b.x - a.x, b.y - a.y);
    let s = Point::new(d.x - c.x, d.y - c.y);
    let denom = cross(r, s);
    if denom.abs() < EPS {
        return false;
    }
    let ca = Point::new(c.x - a.x, c.y - a.y);
    let t = cross(ca, s) / denom;
    let u = cross(ca, r) / denom;
    t > EPS && t < 1.0 - EPS && u > EPS && u < 1.0 - EPS
}

fn cross(a: Point, b: Point) -> f64 {
    a.x * b.y - a.y * b.x
}

fn crossing_angle(a: Point, b: Point) -> f64 {
    let denom = a.x.hypot(a.y) * b.x.hypot(b.y);
    if denom < EPS {
        return 0.0;
    }
    let theta = ((a.x * b.x + a.y * b.y) / denom).clamp(-1.0, 1.0).acos();
    theta.min(std::f64::consts::PI - theta)
}
