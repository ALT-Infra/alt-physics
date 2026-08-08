use std::collections::BTreeMap;

use nalgebra::DVector;

use crate::{
    energy::CompiledProblem,
    geometry::{segments_intersect, Rect},
    LayoutMetrics, NodeId, NodePlacement, Point, Route,
};

const ANGLE_EPS: f64 = 1e-9;

pub(crate) fn measure(
    problem: &CompiledProblem,
    params: &DVector<f64>,
    placements: &BTreeMap<NodeId, NodePlacement>,
    routes: &[Route],
) -> LayoutMetrics {
    let breakdown = problem.breakdown(params);
    let mut overlaps = 0;
    let nodes: Vec<_> = placements.values().copied().collect();
    for left in 0..nodes.len() {
        for right in left + 1..nodes.len() {
            let a = Rect {
                center: nodes[left].center,
                size: nodes[left].size,
            };
            let b = Rect {
                center: nodes[right].center,
                size: nodes[right].size,
            };
            overlaps += usize::from(a.overlaps(b, 0.0));
        }
    }

    let mut crossings = 0;
    let mut minimum_angle: Option<f64> = None;
    for left in 0..routes.len() {
        for right in left + 1..routes.len() {
            let left_edge = problem
                .edges
                .iter()
                .find(|edge| edge.id == routes[left].edge)
                .expect("route refers to compiled edge");
            let right_edge = problem
                .edges
                .iter()
                .find(|edge| edge.id == routes[right].edge)
                .expect("route refers to compiled edge");
            if left_edge.source == right_edge.source
                || left_edge.source == right_edge.target
                || left_edge.target == right_edge.source
                || left_edge.target == right_edge.target
            {
                continue;
            }
            for a in routes[left].points.windows(2) {
                for b in routes[right].points.windows(2) {
                    if let Some((_, angle)) = segments_intersect(a[0], a[1], b[0], b[1]) {
                        crossings += 1;
                        minimum_angle = Some(minimum_angle.map_or(angle, |old| old.min(angle)));
                    }
                }
            }
        }
    }

    let mut incident_vectors: BTreeMap<NodeId, Vec<Point>> = BTreeMap::new();
    for route in routes {
        let edge = problem
            .edges
            .iter()
            .find(|edge| edge.id == route.edge)
            .expect("route refers to compiled edge");
        if let Some(segment) = route.points.windows(2).next() {
            incident_vectors
                .entry(edge.source)
                .or_default()
                .push(Point::new(
                    segment[1].x - segment[0].x,
                    segment[1].y - segment[0].y,
                ));
        }
        if let Some(segment) = route.points.windows(2).last() {
            incident_vectors
                .entry(edge.target)
                .or_default()
                .push(Point::new(
                    segment[0].x - segment[1].x,
                    segment[0].y - segment[1].y,
                ));
        }
    }
    let mut minimum_incident_angle: Option<f64> = None;
    for vectors in incident_vectors.values() {
        for left in 0..vectors.len() {
            for right in left + 1..vectors.len() {
                let angle = vector_angle(vectors[left], vectors[right]);
                minimum_incident_angle =
                    Some(minimum_incident_angle.map_or(angle, |current| current.min(angle)));
            }
        }
    }

    let bounds: Option<(f64, f64, f64, f64)> =
        placements.values().fold(None, |bounds, placement| {
            let left = placement.center.x - placement.size.width * 0.5;
            let right = placement.center.x + placement.size.width * 0.5;
            let top = placement.center.y - placement.size.height * 0.5;
            let bottom = placement.center.y + placement.size.height * 0.5;
            Some(match bounds {
                None => (left, right, top, bottom),
                Some((min_x, max_x, min_y, max_y)) => (
                    min_x.min(left),
                    max_x.max(right),
                    min_y.min(top),
                    max_y.max(bottom),
                ),
            })
        });
    let (drawing_width, drawing_height) = bounds
        .map(|(min_x, max_x, min_y, max_y)| (max_x - min_x, max_y - min_y))
        .unwrap_or_default();

    let total_edge_length = routes
        .iter()
        .flat_map(|route| route.points.windows(2))
        .map(|segment| segment[0].distance(segment[1]))
        .sum();
    let bends = routes
        .iter()
        .map(|route| route.points.len().saturating_sub(2))
        .sum();
    LayoutMetrics {
        energy: breakdown.total,
        stress: breakdown.stress,
        hierarchy_error: breakdown.hierarchy,
        overlaps,
        crossings,
        minimum_crossing_angle_degrees: minimum_angle.map(f64::to_degrees),
        minimum_incident_angle_degrees: minimum_incident_angle.map(f64::to_degrees),
        total_edge_length,
        bends,
        drawing_width,
        drawing_height,
    }
}

fn vector_angle(left: Point, right: Point) -> f64 {
    let denominator = left.x.hypot(left.y) * right.x.hypot(right.y);
    if denominator < ANGLE_EPS {
        return 0.0;
    }
    ((left.x * right.x + left.y * right.y) / denominator)
        .clamp(-1.0, 1.0)
        .acos()
}
