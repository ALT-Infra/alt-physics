use alt_graph_physics::{
    layout, Axis, AxisConstraint, Edge, EdgeKind, LayoutConfig, LayoutInput, Node, Pin, Point,
    Port, Size,
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
        constraints: vec![],
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
        constraints: vec![],
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
        constraints: vec![],
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
        constraints: vec![],
        config: LayoutConfig::default(),
    };
    assert!(layout(&input).is_err());
}

#[test]
fn disconnected_nodes_remain_bounded_and_separated() {
    let input = LayoutInput {
        nodes: vec![node(1), node(2), node(3)],
        edges: vec![],
        constraints: vec![],
        config: LayoutConfig::default(),
    };
    let output = layout(&input).unwrap();
    assert_eq!(output.metrics.overlaps, 0);
    assert!(output
        .placements
        .values()
        .all(|placement| placement.center.x.abs() < 1_000.0 && placement.center.y.abs() < 1_000.0));
}

#[test]
fn axis_constraints_do_not_create_edges_and_hard_separation_survives_projection() {
    let input = LayoutInput {
        nodes: vec![node(1), node(2), node(3)],
        edges: vec![directed(1, 1, 2)],
        constraints: vec![
            AxisConstraint::Position {
                node: 1,
                axis: Axis::Vertical,
                coordinate: 0.0,
                weight: 20.0,
            },
            AxisConstraint::Offset {
                source: 1,
                target: 2,
                axis: Axis::Vertical,
                delta: 180.0,
                weight: 20.0,
            },
            AxisConstraint::Separation {
                before: 1,
                after: 3,
                axis: Axis::Vertical,
                minimum: 100.0,
                weight: 20.0,
            },
        ],
        config: LayoutConfig::default(),
    };
    let output = layout(&input).unwrap();
    assert_eq!(
        output.routes.len(),
        1,
        "constraints manufactured rendered edges"
    );
    assert!(output.placements[&3].center.y - output.placements[&1].center.y >= 100.0 - 1e-6);
    let offset = output.placements[&2].center.y - output.placements[&1].center.y;
    assert!((offset - 180.0).abs() < 4.0, "vertical offset was {offset}");
}

#[test]
fn reports_incident_resolution_and_physical_extent() {
    let fixed = |id, x, y| Node {
        id,
        size: Size::new(100.0, 60.0),
        pin: Pin::Fixed(Point::new(x, y)),
    };
    let input = LayoutInput {
        nodes: vec![
            fixed(1, 0.0, 0.0),
            fixed(2, -200.0, 200.0),
            fixed(3, 200.0, 200.0),
        ],
        edges: vec![directed(1, 1, 2), directed(2, 1, 3)],
        constraints: vec![],
        config: LayoutConfig::default(),
    };
    let output = layout(&input).unwrap();
    let angle = output.metrics.minimum_incident_angle_degrees.unwrap();
    assert!(angle > 70.0, "incident angle was {angle}");
    assert_eq!(output.metrics.drawing_width, 500.0);
    assert_eq!(output.metrics.drawing_height, 260.0);
}

#[test]
fn dense_overlapping_lead_pools_remain_hierarchical_and_measurable() {
    let mut edges = Vec::new();
    let mut edge_id = 0;
    let mut push = |source, target, kind, weight| {
        edge_id += 1;
        edges.push(Edge {
            id: edge_id,
            source,
            target,
            kind,
            ideal_length: if matches!(kind, EdgeKind::Association) {
                250.0
            } else {
                230.0
            },
            weight,
            source_port: Port::Free,
            target_port: Port::Free,
        });
    };
    for lead in 2..=5 {
        push(
            1,
            lead,
            EdgeKind::Directed {
                target_delta: 190.0,
            },
            1.4,
        );
    }
    for (lead, contributor) in [
        (2, 7),
        (2, 10),
        (2, 13),
        (3, 6),
        (3, 11),
        (4, 8),
        (4, 11),
        (5, 10),
        (5, 9),
    ] {
        push(
            lead,
            contributor,
            EdgeKind::Directed {
                target_delta: 190.0,
            },
            1.0,
        );
    }
    for (lead, contributor) in [
        (2, 6),
        (2, 9),
        (2, 12),
        (3, 13),
        (3, 12),
        (3, 8),
        (4, 6),
        (4, 7),
        (4, 12),
        (5, 7),
        (5, 13),
        (5, 8),
    ] {
        push(
            lead,
            contributor,
            EdgeKind::Directed {
                target_delta: 190.0,
            },
            0.9,
        );
    }
    let mut constraints = vec![AxisConstraint::Position {
        node: 1,
        axis: Axis::Vertical,
        coordinate: 0.0,
        weight: 30.0,
    }];
    for lead in 2..=5 {
        constraints.push(AxisConstraint::Offset {
            source: 1,
            target: lead,
            axis: Axis::Vertical,
            delta: 190.0,
            weight: 16.0,
        });
        constraints.push(AxisConstraint::Separation {
            before: 1,
            after: lead,
            axis: Axis::Vertical,
            minimum: 150.0,
            weight: 24.0,
        });
        if lead != 2 {
            constraints.push(AxisConstraint::Alignment {
                first: 2,
                second: lead,
                axis: Axis::Vertical,
                weight: 20.0,
            });
        }
    }
    for contributor in 6..=13 {
        constraints.push(AxisConstraint::Separation {
            before: 1,
            after: contributor,
            axis: Axis::Vertical,
            minimum: 145.0,
            weight: 24.0,
        });
    }
    for edge in &edges {
        match edge.kind {
            EdgeKind::Directed { .. } if edge.source != 1 => {
                constraints.push(AxisConstraint::Separation {
                    before: edge.source,
                    after: edge.target,
                    axis: Axis::Vertical,
                    minimum: 150.0,
                    weight: 18.0,
                });
            }
            EdgeKind::Association => constraints.push(AxisConstraint::Offset {
                source: edge.source,
                target: edge.target,
                axis: Axis::Vertical,
                delta: 0.0,
                weight: 12.0,
            }),
            EdgeKind::Directed { .. } => {}
        }
    }
    let output = layout(&LayoutInput {
        nodes: (1..=13)
            .map(|id| Node {
                id,
                size: if id == 1 {
                    Size::new(180.0, 72.0)
                } else {
                    Size::new(220.0, 92.0)
                },
                pin: Pin::Free,
            })
            .collect(),
        edges,
        constraints,
        config: LayoutConfig {
            max_iterations: 360,
            clearance: 55.0,
            route_clearance: 14.0,
            hierarchy_weight: 2.8,
            crossing_weight: 0.3,
            ..LayoutConfig::default()
        },
    })
    .unwrap();
    assert_eq!(output.metrics.overlaps, 0);
    assert!(output.placements[&1].center.y < output.placements[&2].center.y);
    assert!(output.placements[&1].center.y < output.placements[&5].center.y);
    for lead in 3..=5 {
        assert_eq!(
            output.placements[&2].center.y, output.placements[&lead].center.y,
            "exact Lead rank was broken"
        );
    }
    assert!(output.metrics.drawing_width > 0.0);
    assert!(output.metrics.drawing_height > 0.0);
    eprintln!("dense ALT audit: {:#?}", output.metrics);
}
