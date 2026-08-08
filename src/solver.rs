use std::collections::BTreeMap;

use argmin::{
    core::{Executor, State},
    solver::{linesearch::MoreThuenteLineSearch, quasinewton::LBFGS},
};
use nalgebra::DVector;

use crate::{
    energy::CompiledProblem, geometry::Rect, initialize::initialize, metrics::measure,
    routing::route_edges, Axis, AxisConstraint, LayoutError, LayoutInput, LayoutOutput, NodeId,
    NodePlacement, Pin, Point, SolverDiagnostics,
};

pub fn layout(input: &LayoutInput) -> Result<LayoutOutput, LayoutError> {
    let attempts = if input.nodes.len() < 3
        || input
            .nodes
            .iter()
            .all(|node| matches!(node.pin, Pin::Fixed(_)))
    {
        1
    } else {
        input.config.restarts
    };
    let mut best: Option<(usize, LayoutOutput)> = None;
    for restart in 0..attempts {
        let mut candidate_input = input.clone();
        candidate_input.config.seed = restart_seed(input.config.seed, restart);
        candidate_input.config.restarts = 1;
        let candidate = layout_once(&candidate_input)?;
        if best
            .as_ref()
            .is_none_or(|(_, current)| output_is_better(&candidate, current))
        {
            best = Some((restart, candidate));
        }
    }
    let (selected_restart, mut output) = best.expect("at least one restart is required");
    output.diagnostics.attempted_restarts = attempts;
    output.diagnostics.selected_restart = selected_restart;
    Ok(output)
}

fn layout_once(input: &LayoutInput) -> Result<LayoutOutput, LayoutError> {
    let problem = CompiledProblem::new(input)?;
    if problem.nodes.is_empty() {
        return Ok(LayoutOutput {
            placements: BTreeMap::new(),
            routes: Vec::new(),
            metrics: Default::default(),
            diagnostics: SolverDiagnostics {
                termination: "empty graph".into(),
                ..Default::default()
            },
        });
    }

    let initial = initialize(&problem);
    let initial_params = problem.params_from_positions(&initial);
    let (mut params, iterations, termination) = if initial_params.is_empty() {
        (initial_params, 0, "all nodes fixed".to_owned())
    } else {
        optimize(&problem, initial_params)?
    };
    let mut positions = problem.positions(&params);
    let projected_pairs = project_geometry(&problem, &mut positions);
    params = problem.params_from_positions(&positions);

    // A short polish restores spring and hierarchy quality after exact projection.
    if !params.is_empty() && projected_pairs > 0 {
        let polished = optimize_with_limit(
            &problem,
            params.clone(),
            problem.config.max_iterations.min(80),
        )?;
        params = polished.0;
        positions = problem.positions(&params);
        project_geometry(&problem, &mut positions);
        params = problem.params_from_positions(&positions);
    }

    let placements: BTreeMap<NodeId, NodePlacement> = problem
        .nodes
        .iter()
        .zip(positions.iter())
        .map(|(node, center)| {
            (
                node.id,
                NodePlacement {
                    center: *center,
                    size: node.size,
                },
            )
        })
        .collect();
    let (routes, routed_obstacles) =
        route_edges(&problem.edges, &placements, problem.config.route_clearance);
    let metrics = measure(&problem, &params, &placements, &routes);
    Ok(LayoutOutput {
        placements,
        routes,
        metrics,
        diagnostics: SolverDiagnostics {
            iterations,
            termination,
            attempted_restarts: 1,
            selected_restart: 0,
            projected_pairs,
            routed_obstacles,
        },
    })
}

fn restart_seed(seed: u64, restart: usize) -> u64 {
    if restart == 0 {
        return seed;
    }
    let mut value = seed.wrapping_add((restart as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn output_is_better(candidate: &LayoutOutput, current: &LayoutOutput) -> bool {
    let candidate_angle = candidate
        .metrics
        .minimum_crossing_angle_degrees
        .unwrap_or(90.0);
    let current_angle = current
        .metrics
        .minimum_crossing_angle_degrees
        .unwrap_or(90.0);
    let candidate_incident_angle = candidate
        .metrics
        .minimum_incident_angle_degrees
        .unwrap_or(180.0);
    let current_incident_angle = current
        .metrics
        .minimum_incident_angle_degrees
        .unwrap_or(180.0);
    candidate
        .metrics
        .overlaps
        .cmp(&current.metrics.overlaps)
        .then_with(|| candidate.metrics.crossings.cmp(&current.metrics.crossings))
        .then_with(|| current_angle.total_cmp(&candidate_angle))
        .then_with(|| current_incident_angle.total_cmp(&candidate_incident_angle))
        .then_with(|| candidate.metrics.bends.cmp(&current.metrics.bends))
        .then_with(|| {
            candidate
                .metrics
                .total_edge_length
                .total_cmp(&current.metrics.total_edge_length)
        })
        .then_with(|| {
            candidate
                .metrics
                .hierarchy_error
                .total_cmp(&current.metrics.hierarchy_error)
        })
        .then_with(|| candidate.metrics.stress.total_cmp(&current.metrics.stress))
        .then_with(|| candidate.metrics.energy.total_cmp(&current.metrics.energy))
        .is_lt()
}

fn optimize(
    problem: &CompiledProblem,
    initial: DVector<f64>,
) -> Result<(DVector<f64>, u64, String), LayoutError> {
    optimize_with_limit(problem, initial, problem.config.max_iterations)
}

fn optimize_with_limit(
    problem: &CompiledProblem,
    initial: DVector<f64>,
    iterations: u64,
) -> Result<(DVector<f64>, u64, String), LayoutError> {
    let line_search = MoreThuenteLineSearch::new()
        .with_c(1e-4, 0.9)
        .map_err(|error| LayoutError::Optimization(error.to_string()))?;
    let solver = LBFGS::new(line_search, problem.config.history_size)
        .with_tolerance_grad(problem.config.gradient_tolerance)
        .map_err(|error| LayoutError::Optimization(error.to_string()))?;
    let result = Executor::new(problem.clone(), solver)
        .configure(|state| state.param(initial).max_iters(iterations))
        .timer(false)
        .run()
        .map_err(|error| LayoutError::Optimization(error.to_string()))?;
    let state = result.state();
    let params = state
        .get_best_param()
        .or_else(|| state.get_param())
        .cloned()
        .ok_or_else(|| LayoutError::Optimization("optimizer returned no parameters".into()))?;
    Ok((
        params,
        state.get_iter(),
        state.get_termination_status().to_string(),
    ))
}

fn project_geometry(problem: &CompiledProblem, positions: &mut [Point]) -> usize {
    let mut projected = 0;
    let horizontal_alignments = alignment_components(problem, Axis::Horizontal);
    let vertical_alignments = alignment_components(problem, Axis::Vertical);
    for _ in 0..problem.config.projection_passes {
        let mut changed = project_axis_alignments(problem, positions, &mut projected);
        changed |= project_axis_separations(problem, positions, &mut projected);
        for left in 0..positions.len() {
            for right in left + 1..positions.len() {
                let a = Rect {
                    center: positions[left],
                    size: problem.nodes[left].size,
                };
                let b = Rect {
                    center: positions[right],
                    size: problem.nodes[right].size,
                };
                if !a.overlaps(b, problem.config.clearance) {
                    continue;
                }
                let dx = positions[right].x - positions[left].x;
                let dy = positions[right].y - positions[left].y;
                let need_x =
                    (a.size.width + b.size.width) * 0.5 + problem.config.clearance - dx.abs();
                let need_y =
                    (a.size.height + b.size.height) * 0.5 + problem.config.clearance - dy.abs();
                let same_horizontal = horizontal_alignments[left] == horizontal_alignments[right];
                let same_vertical = vertical_alignments[left] == vertical_alignments[right];
                let separate_horizontally = if same_vertical && !same_horizontal {
                    true
                } else if same_horizontal && !same_vertical {
                    false
                } else {
                    need_x <= need_y
                };
                let (move_x, move_y) = if separate_horizontally {
                    (
                        need_x * stable_sign(dx, problem.nodes[left].id, problem.nodes[right].id),
                        0.0,
                    )
                } else {
                    (
                        0.0,
                        need_y * stable_sign(dy, problem.nodes[left].id, problem.nodes[right].id),
                    )
                };
                let left_fixed = matches!(problem.nodes[left].pin, Pin::Fixed(_));
                let right_fixed = matches!(problem.nodes[right].pin, Pin::Fixed(_));
                match (left_fixed, right_fixed) {
                    (true, true) => continue,
                    (true, false) => {
                        positions[right].x += move_x;
                        positions[right].y += move_y;
                    }
                    (false, true) => {
                        positions[left].x -= move_x;
                        positions[left].y -= move_y;
                    }
                    (false, false) => {
                        positions[left].x -= move_x * 0.5;
                        positions[left].y -= move_y * 0.5;
                        positions[right].x += move_x * 0.5;
                        positions[right].y += move_y * 0.5;
                    }
                }
                projected += 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Inequality projection may move members of one equality component by
    // slightly different amounts on the final pass. Equality is the stricter
    // contract, and aligned overlap pairs were already separated along the
    // orthogonal axis above, so finish by restoring it exactly.
    project_axis_alignments(problem, positions, &mut projected);
    projected
}

fn alignment_components(problem: &CompiledProblem, axis: Axis) -> Vec<usize> {
    let mut parents: Vec<_> = (0..problem.nodes.len()).collect();
    for constraint in &problem.constraints {
        let AxisConstraint::Alignment {
            first,
            second,
            axis: constraint_axis,
            ..
        } = *constraint
        else {
            continue;
        };
        if constraint_axis != axis {
            continue;
        }
        let first = find_component(&parents, problem.index[&first]);
        let second = find_component(&parents, problem.index[&second]);
        let representative = first.min(second);
        parents[first] = representative;
        parents[second] = representative;
    }
    (0..parents.len())
        .map(|index| find_component(&parents, index))
        .collect()
}

fn find_component(parents: &[usize], mut index: usize) -> usize {
    while parents[index] != index {
        index = parents[index];
    }
    index
}

fn project_axis_alignments(
    problem: &CompiledProblem,
    positions: &mut [Point],
    projected: &mut usize,
) -> bool {
    let mut changed = false;
    for axis in [Axis::Horizontal, Axis::Vertical] {
        let components = alignment_components(problem, axis);
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (index, component) in components.into_iter().enumerate() {
            groups.entry(component).or_default().push(index);
        }
        for group in groups.values().filter(|group| group.len() > 1) {
            let target = group
                .iter()
                .copied()
                .find(|&index| matches!(problem.nodes[index].pin, Pin::Fixed(_)))
                .map(|index| coordinate(positions[index], axis))
                .unwrap_or_else(|| {
                    group
                        .iter()
                        .map(|&index| coordinate(positions[index], axis))
                        .sum::<f64>()
                        / group.len() as f64
                });
            for &index in group {
                if matches!(problem.nodes[index].pin, Pin::Fixed(_)) {
                    continue;
                }
                let delta = target - coordinate(positions[index], axis);
                if delta.abs() <= 1e-8 {
                    continue;
                }
                move_along(&mut positions[index], axis, delta);
                *projected += 1;
                changed = true;
            }
        }
    }
    changed
}

fn project_axis_separations(
    problem: &CompiledProblem,
    positions: &mut [Point],
    projected: &mut usize,
) -> bool {
    let mut changed = false;
    for constraint in &problem.constraints {
        let AxisConstraint::Separation {
            before,
            after,
            axis,
            minimum,
            ..
        } = *constraint
        else {
            continue;
        };
        let before = problem.index[&before];
        let after = problem.index[&after];
        let current = coordinate(positions[after], axis) - coordinate(positions[before], axis);
        let violation = minimum - current;
        if violation <= 1e-8 {
            continue;
        }
        let before_fixed = matches!(problem.nodes[before].pin, Pin::Fixed(_));
        let after_fixed = matches!(problem.nodes[after].pin, Pin::Fixed(_));
        match (before_fixed, after_fixed) {
            (true, true) => continue,
            (true, false) => move_along(&mut positions[after], axis, violation),
            (false, true) => move_along(&mut positions[before], axis, -violation),
            (false, false) => {
                move_along(&mut positions[before], axis, -violation * 0.5);
                move_along(&mut positions[after], axis, violation * 0.5);
            }
        }
        *projected += 1;
        changed = true;
    }
    changed
}

fn coordinate(point: Point, axis: Axis) -> f64 {
    match axis {
        Axis::Horizontal => point.x,
        Axis::Vertical => point.y,
    }
}

fn move_along(point: &mut Point, axis: Axis, amount: f64) {
    match axis {
        Axis::Horizontal => point.x += amount,
        Axis::Vertical => point.y += amount,
    }
}

fn stable_sign(value: f64, left: u64, right: u64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else if left < right {
        1.0
    } else {
        -1.0
    }
}
