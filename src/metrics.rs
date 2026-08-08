use std::collections::BTreeMap;

use nalgebra::DVector;

use crate::{
    energy::CompiledProblem,
    geometry::{segments_intersect, Rect},
    LayoutMetrics, NodeId, NodePlacement, Route,
};

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
        total_edge_length,
        bends,
    }
}
