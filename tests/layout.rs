use alt_graph_physics::{
    layout, Edge, EdgeKind, LayoutConfig, LayoutInput, Node, Pin, Point, Port, Size,
};

fn node(id: u64) -> Node {
    Node {
        id,
        size: Size::new(120.0, 64.0),
        pin: Pin::Free,
    }
}

fn directed(id: u64, source: u64, target: u64) -> Edge {
    Edge {
        id,
        source,
        target,
        kind: EdgeKind::Directed {
            target_delta: 150.0,
        },
        ideal_length: 190.0,
        weight: 1.0,
        source_port: Port::Free,
        target_port: Port::Free,
    }
}

#[test]
fn mixed_directed_and_peer_graph_is_deterministic_and_clear() {
    let input = LayoutInput {
        nodes: (1..=7).map(node).collect(),
        edges: vec![
            directed(1, 1, 2),
            directed(2, 1, 3),
            directed(3, 2, 4),
            directed(4, 2, 5),
            directed(5, 3, 5),
            directed(6, 3, 6),
            Edge {
                id: 7,
                source: 2,
                target: 7,
                kind: EdgeKind::Association,
                ideal_length: 180.0,
                weight: 0.8,
                source_port: Port::Free,
                target_port: Port::Free,
            },
        ],
        config: LayoutConfig::default(),
    };
    let first = layout(&input).unwrap();
    let second = layout(&input).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.metrics.overlaps, 0);
    assert!(first.placements[&2].center.y > first.placements[&1].center.y);
    assert!(first.placements[&3].center.y > first.placements[&1].center.y);
    assert_eq!(first.routes.len(), input.edges.len());
}

#[test]
fn fixed_positions_are_exact() {
    let mut left = node(1);
    left.pin = Pin::Fixed(Point::new(-300.0, 20.0));
    let input = LayoutInput {
        nodes: vec![left, node(2)],
        edges: vec![directed(1, 1, 2)],
        config: LayoutConfig::default(),
    };
    let output = layout(&input).unwrap();
    assert_eq!(output.placements[&1].center, Point::new(-300.0, 20.0));
}

#[test]
fn route_avoids_a_fixed_rectangular_obstacle() {
    let fixed = |id, x| Node {
        id,
        size: Size::new(100.0, 100.0),
        pin: Pin::Fixed(Point::new(x, 0.0)),
    };
    let input = LayoutInput {
        nodes: vec![fixed(1, -300.0), fixed(2, 300.0), fixed(3, 0.0)],
        edges: vec![Edge {
            id: 1,
            source: 1,
            target: 2,
            kind: EdgeKind::Association,
            ideal_length: 600.0,
            weight: 1.0,
            source_port: Port::Free,
            target_port: Port::Free,
        }],
        config: LayoutConfig::default(),
    };
    let output = layout(&input).unwrap();
    assert!(output.routes[0].points.len() > 2);
    assert_eq!(output.diagnostics.routed_obstacles, 1);
}

#[test]
fn invalid_endpoint_is_rejected() {
    let input = LayoutInput {
        nodes: vec![node(1)],
        edges: vec![directed(1, 1, 99)],
        config: LayoutConfig::default(),
    };
    assert!(layout(&input).is_err());
}

#[test]
fn disconnected_nodes_remain_bounded_and_separated() {
    let input = LayoutInput {
        nodes: vec![node(1), node(2), node(3)],
        edges: vec![],
        config: LayoutConfig::default(),
    };
    let output = layout(&input).unwrap();
    assert_eq!(output.metrics.overlaps, 0);
    assert!(output
        .placements
        .values()
        .all(|placement| placement.center.x.abs() < 1_000.0 && placement.center.y.abs() < 1_000.0));
}
