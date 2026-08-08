use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    geometry::{boundary_point, segment_hits_rect, Rect, EPS},
    Edge, EdgeId, NodeId, NodePlacement, Point, Port, Route, Side,
};

type ResolvedPorts = BTreeMap<(EdgeId, bool), Option<(Side, Option<f64>)>>;

pub(crate) fn route_edges(
    edges: &[Edge],
    placements: &BTreeMap<NodeId, NodePlacement>,
    clearance: f64,
) -> (Vec<Route>, usize) {
    let resolved_ports = resolve_ports(edges, placements);
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
        let start = boundary_point(
            source_rect,
            target.center,
            resolved_ports.get(&(edge.id, true)).copied().flatten(),
        );
        let end = boundary_point(
            target_rect,
            source.center,
            resolved_ports.get(&(edge.id, false)).copied().flatten(),
        );
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

/// Resolve free and side-only endpoints as a set, not edge-by-edge. Distinct,
/// monotonically ordered boundary positions prevent incident routes from
/// leaving a node on top of one another and realize the local two-layer
/// crossing minimum at each rectangle boundary.
fn resolve_ports(edges: &[Edge], placements: &BTreeMap<NodeId, NodePlacement>) -> ResolvedPorts {
    #[derive(Clone, Copy)]
    struct Endpoint {
        edge: EdgeId,
        source: bool,
        side: Side,
        toward: Point,
        fixed_offset: Option<f64>,
    }

    let mut endpoints = Vec::with_capacity(edges.len() * 2);
    for edge in edges {
        let source = placements[&edge.source];
        let target = placements[&edge.target];
        for (source_endpoint, node, toward, port) in [
            (true, source, target.center, edge.source_port),
            (false, target, source.center, edge.target_port),
        ] {
            let rect = Rect {
                center: node.center,
                size: node.size,
            };
            let (side, fixed_offset) = match port {
                Port::Fixed { side, offset } => (side, Some(offset.clamp(-1.0, 1.0))),
                Port::Side(side) => (side, None),
                Port::Free => (natural_side(rect, toward), None),
            };
            endpoints.push(Endpoint {
                edge: edge.id,
                source: source_endpoint,
                side,
                toward,
                fixed_offset,
            });
        }
    }

    let mut result = BTreeMap::new();
    let mut groups: BTreeMap<(NodeId, u8), Vec<(usize, Endpoint)>> = BTreeMap::new();
    for (index, endpoint) in endpoints.iter().copied().enumerate() {
        let edge = edges
            .iter()
            .find(|edge| edge.id == endpoint.edge)
            .expect("resolved endpoint refers to an edge");
        let node = if endpoint.source {
            edge.source
        } else {
            edge.target
        };
        if let Some(offset) = endpoint.fixed_offset {
            result.insert(
                (endpoint.edge, endpoint.source),
                Some((endpoint.side, Some(offset))),
            );
        } else {
            groups
                .entry((node, side_order(endpoint.side)))
                .or_default()
                .push((index, endpoint));
        }
    }

    for ((node_id, _), mut group) in groups {
        let node = placements[&node_id];
        group.sort_by(|(_, left), (_, right)| {
            side_coordinate(left.side, left.toward)
                .total_cmp(&side_coordinate(right.side, right.toward))
                .then_with(|| left.edge.cmp(&right.edge))
                .then_with(|| left.source.cmp(&right.source))
        });
        let count = group.len();
        for (ordinal, (_, endpoint)) in group.into_iter().enumerate() {
            let offset = if count == 1 {
                natural_offset(
                    endpoint.side,
                    Rect {
                        center: node.center,
                        size: node.size,
                    },
                    endpoint.toward,
                )
            } else {
                -0.76 + 1.52 * ordinal as f64 / (count - 1) as f64
            };
            result.insert(
                (endpoint.edge, endpoint.source),
                Some((endpoint.side, Some(offset))),
            );
        }
    }
    result
}

fn natural_side(rect: Rect, toward: Point) -> Side {
    let delta = Point::new(toward.x - rect.center.x, toward.y - rect.center.y);
    let horizontal = if delta.x.abs() < EPS {
        f64::INFINITY
    } else {
        rect.size.width * 0.5 / delta.x.abs()
    };
    let vertical = if delta.y.abs() < EPS {
        f64::INFINITY
    } else {
        rect.size.height * 0.5 / delta.y.abs()
    };
    if horizontal < vertical {
        if delta.x < 0.0 {
            Side::Left
        } else {
            Side::Right
        }
    } else if delta.y < 0.0 {
        Side::Top
    } else {
        Side::Bottom
    }
}

fn natural_offset(side: Side, rect: Rect, toward: Point) -> f64 {
    match side {
        Side::Top | Side::Bottom => {
            ((toward.x - rect.center.x) / (rect.size.width * 0.5).max(EPS)).clamp(-0.86, 0.86)
        }
        Side::Left | Side::Right => {
            ((toward.y - rect.center.y) / (rect.size.height * 0.5).max(EPS)).clamp(-0.86, 0.86)
        }
    }
}

fn side_coordinate(side: Side, toward: Point) -> f64 {
    match side {
        Side::Top | Side::Bottom => toward.x,
        Side::Left | Side::Right => toward.y,
    }
}

fn side_order(side: Side) -> u8 {
    match side {
        Side::Top => 0,
        Side::Right => 1,
        Side::Bottom => 2,
        Side::Left => 3,
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
