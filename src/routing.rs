use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    geometry::{boundary_point, segment_hits_rect, Rect, EPS},
    Edge, NodeId, NodePlacement, Point, Port, Route, Side,
};

pub(crate) fn route_edges(
    edges: &[Edge],
    placements: &BTreeMap<NodeId, NodePlacement>,
    clearance: f64,
) -> (Vec<Route>, usize) {
    let mut routes = Vec::with_capacity(edges.len());
    let mut obstacle_routes = 0;
    for edge in edges {
        let source = placements[&edge.source];
        let target = placements[&edge.target];
        let source_rect = Rect {
            center: source.center,
            size: source.size,
        };
        let target_rect = Rect {
            center: target.center,
            size: target.size,
        };
        let start = boundary_point(source_rect, target.center, port_side(edge.source_port));
        let end = boundary_point(target_rect, source.center, port_side(edge.target_port));
        let obstacles: Vec<_> = placements
            .iter()
            .filter(|(id, _)| **id != edge.source && **id != edge.target)
            .map(|(_, placement)| {
                Rect {
                    center: placement.center,
                    size: placement.size,
                }
                .expanded(clearance)
            })
            .collect();
        let points = shortest_visible_route(start, end, &obstacles);
        if points.len() > 2 {
            obstacle_routes += 1;
        }
        routes.push(Route {
            edge: edge.id,
            points: simplify(points),
        });
    }
    (routes, obstacle_routes)
}

fn port_side(port: Port) -> Option<(Side, Option<f64>)> {
    match port {
        Port::Free => None,
        Port::Side(side) => Some((side, None)),
        Port::Fixed { side, offset } => Some((side, Some(offset.clamp(-1.0, 1.0)))),
    }
}

fn shortest_visible_route(start: Point, end: Point, obstacles: &[Rect]) -> Vec<Point> {
    if visible(start, end, obstacles) {
        return vec![start, end];
    }
    let mut vertices = vec![start, end];
    for obstacle in obstacles {
        vertices.extend(obstacle.corners());
    }
    let count = vertices.len();
    let mut distance = vec![f64::INFINITY; count];
    let mut previous = vec![None; count];
    let mut visited = vec![false; count];
    distance[0] = 0.0;
    for _ in 0..count {
        let current = (0..count)
            .filter(|&index| !visited[index])
            .min_by(|&a, &b| distance[a].total_cmp(&distance[b]).then_with(|| a.cmp(&b)));
        let Some(current) = current else {
            break;
        };
        if !distance[current].is_finite() || current == 1 {
            break;
        }
        visited[current] = true;
        for next in 0..count {
            if next == current
                || visited[next]
                || !visible(vertices[current], vertices[next], obstacles)
            {
                continue;
            }
            let candidate = distance[current] + vertices[current].distance(vertices[next]);
            let order = candidate.total_cmp(&distance[next]);
            if order == Ordering::Less
                || (order == Ordering::Equal && previous[next].is_none_or(|old| current < old))
            {
                distance[next] = candidate;
                previous[next] = Some(current);
            }
        }
    }
    if !distance[1].is_finite() {
        return vec![start, end];
    }
    let mut path = vec![end];
    let mut cursor = 1;
    while let Some(parent) = previous[cursor] {
        path.push(vertices[parent]);
        cursor = parent;
    }
    path.reverse();
    path
}

fn visible(a: Point, b: Point, obstacles: &[Rect]) -> bool {
    if a.distance(b) < EPS {
        return false;
    }
    !obstacles
        .iter()
        .any(|obstacle| segment_hits_rect(a, b, *obstacle))
}

fn simplify(points: Vec<Point>) -> Vec<Point> {
    if points.len() < 3 {
        return points;
    }
    let mut simplified = Vec::with_capacity(points.len());
    for point in points {
        while simplified.len() >= 2 {
            let a: Point = simplified[simplified.len() - 2];
            let b: Point = simplified[simplified.len() - 1];
            let ab = Point::new(b.x - a.x, b.y - a.y);
            let bp = Point::new(point.x - b.x, point.y - b.y);
            if (ab.x * bp.y - ab.y * bp.x).abs() > 1e-7 {
                break;
            }
            simplified.pop();
        }
        simplified.push(point);
    }
    simplified
}
