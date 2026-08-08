use std::{env, fmt::Write as _, fs};

use alt_graph_physics::{
    layout, Axis, AxisConstraint, Edge, EdgeKind, LayoutConfig, LayoutInput, LayoutOutput, Node,
    Pin, Point, Port, Size,
};

fn main() {
    let output_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "target/dense-layout-audit.svg".to_owned());
    let variants = [
        ("balanced", 0.30, 2.8, 4),
        ("crossing 0.8", 0.80, 2.8, 6),
        ("crossing 1.6", 1.60, 2.8, 8),
        ("crossing 3.2", 3.20, 2.8, 8),
        ("flow 4.2", 1.60, 4.2, 8),
        ("flow 6.0", 1.60, 6.0, 8),
    ];
    let results: Vec<_> = variants
        .into_iter()
        .map(|(name, crossing_weight, hierarchy_weight, restarts)| {
            let mut input = dense_fixture();
            input.config.crossing_weight = crossing_weight;
            input.config.hierarchy_weight = hierarchy_weight;
            input.config.restarts = restarts;
            let output = layout(&input).expect("dense audit layout");
            eprintln!("{name}: {:#?}", output.metrics);
            (name, input, output)
        })
        .collect();
    fs::create_dir_all(
        std::path::Path::new(&output_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
    )
    .expect("create audit output directory");
    fs::write(&output_path, render_contact_sheet(&results)).expect("write audit SVG");
    println!("{output_path}");
}

fn dense_fixture() -> LayoutInput {
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
    LayoutInput {
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
    }
}

fn render_contact_sheet(results: &[(&str, LayoutInput, LayoutOutput)]) -> String {
    const PANEL_W: f64 = 720.0;
    const PANEL_H: f64 = 500.0;
    const COLS: usize = 2;
    let rows = results.len().div_ceil(COLS);
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><rect width="100%" height="100%" fill="#0d0f12"/><style>text{{font-family:ui-monospace,monospace}} .route{{fill:none;stroke-linecap:round;stroke-linejoin:round}} .node{{stroke-width:1.5}} </style>"##,
        PANEL_W * COLS as f64,
        PANEL_H * rows as f64,
        PANEL_W * COLS as f64,
        PANEL_H * rows as f64
    );
    for (index, (name, input, output)) in results.iter().enumerate() {
        let origin_x = (index % COLS) as f64 * PANEL_W;
        let origin_y = (index / COLS) as f64 * PANEL_H;
        let margin = 34.0;
        let header = 66.0;
        let scale = ((PANEL_W - margin * 2.0) / output.metrics.drawing_width)
            .min((PANEL_H - header - margin) / output.metrics.drawing_height)
            .min(0.48);
        let min_x = output
            .placements
            .values()
            .map(|node| node.center.x - node.size.width * 0.5)
            .fold(f64::INFINITY, f64::min);
        let min_y = output
            .placements
            .values()
            .map(|node| node.center.y - node.size.height * 0.5)
            .fold(f64::INFINITY, f64::min);
        let transform = |point: Point| {
            (
                origin_x + margin + (point.x - min_x) * scale,
                origin_y + header + (point.y - min_y) * scale,
            )
        };
        let _ = write!(
            svg,
            r##"<g><rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#26303a"/><text x="{}" y="{}" fill="#eef2f6" font-size="16">{}</text><text x="{}" y="{}" fill="#8e9bab" font-size="11">cross {} · angle {:.1}° · incident {:.1}° · bends {} · {:.0}×{:.0}</text>"##,
            origin_x + 0.5,
            origin_y + 0.5,
            PANEL_W - 1.0,
            PANEL_H - 1.0,
            origin_x + 18.0,
            origin_y + 24.0,
            name,
            origin_x + 18.0,
            origin_y + 45.0,
            output.metrics.crossings,
            output
                .metrics
                .minimum_crossing_angle_degrees
                .unwrap_or(90.0),
            output
                .metrics
                .minimum_incident_angle_degrees
                .unwrap_or(180.0),
            output.metrics.bends,
            output.metrics.drawing_width,
            output.metrics.drawing_height,
        );
        for route in &output.routes {
            let edge = input
                .edges
                .iter()
                .find(|edge| edge.id == route.edge)
                .unwrap();
            let peer = edge.id > 13;
            let color = if peer { "#e6b94b" } else { "#4ab9df" };
            let dash = if peer {
                r#" stroke-dasharray="6 4""#
            } else {
                ""
            };
            let points = route
                .points
                .iter()
                .map(|point| {
                    let (x, y) = transform(*point);
                    format!("{x:.2},{y:.2}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            let _ = write!(
                svg,
                r#"<polyline class="route" points="{}" stroke="{}" stroke-opacity="0.68" stroke-width="1.3"{}/>"#,
                points, color, dash
            );
        }
        for (&id, node) in &output.placements {
            let (x, y) = transform(Point::new(
                node.center.x - node.size.width * 0.5,
                node.center.y - node.size.height * 0.5,
            ));
            let (fill, stroke) = if id == 1 {
                ("#191625", "#b284ff")
            } else if id <= 5 {
                ("#121c2b", "#63a4ff")
            } else {
                ("#111f1c", "#52d39e")
            };
            let _ = write!(
                svg,
                r#"<rect class="node" x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="5" fill="{}" stroke="{}"/><text x="{:.2}" y="{:.2}" fill="{}" font-size="10">{}</text>"#,
                x,
                y,
                node.size.width * scale,
                node.size.height * scale,
                fill,
                stroke,
                x + 7.0,
                y + 15.0,
                stroke,
                id
            );
        }
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");
    svg
}
