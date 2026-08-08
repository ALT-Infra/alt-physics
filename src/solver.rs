use std::collections::BTreeMap;

use argmin::{
    core::{Executor, State},
    solver::{linesearch::MoreThuenteLineSearch, quasinewton::LBFGS},
};
use nalgebra::DVector;

use crate::{
    energy::CompiledProblem, geometry::Rect, initialize::initialize, metrics::measure,
    routing::route_edges, LayoutError, LayoutInput, LayoutOutput, NodeId, NodePlacement, Pin,
    Point, SolverDiagnostics,
};

pub fn layout(input: &LayoutInput) -> Result<LayoutOutput, LayoutError> {
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
    let projected_pairs = project_non_overlap(&problem, &mut positions);
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
        project_non_overlap(&problem, &mut positions);
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
            projected_pairs,
            routed_obstacles,
        },
    })
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

fn project_non_overlap(problem: &CompiledProblem, positions: &mut [Point]) -> usize {
    let mut projected = 0;
    for _ in 0..problem.config.projection_passes {
        let mut changed = false;
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
                let (move_x, move_y) = if need_x <= need_y {
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
    projected
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
