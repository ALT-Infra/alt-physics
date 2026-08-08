use std::collections::{BTreeMap, BTreeSet};

use argmin::core::{CostFunction, Error as ArgminError, Gradient};
use nalgebra::DVector;

use crate::geometry::segments_intersect;
use crate::{
    Axis, AxisConstraint, Edge, EdgeKind, LayoutConfig, LayoutError, LayoutInput, Node, NodeId,
    Pin, Point,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EnergyBreakdown {
    pub total: f64,
    pub stress: f64,
    pub hierarchy: f64,
}

#[derive(Clone)]
pub(crate) struct CompiledProblem {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub constraints: Vec<AxisConstraint>,
    pub index: BTreeMap<NodeId, usize>,
    pub free_slot: Vec<Option<usize>>,
    pub fixed: Vec<Option<Point>>,
    pub config: LayoutConfig,
    pub repulsion_scale: f64,
}

impl CompiledProblem {
    pub(crate) fn new(input: &LayoutInput) -> Result<Self, LayoutError> {
        validate_config(&input.config)?;
        let mut nodes = input.nodes.clone();
        nodes.sort_by_key(|node| node.id);
        let mut node_ids = BTreeSet::new();
        for node in &nodes {
            if !node_ids.insert(node.id) {
                return Err(LayoutError::DuplicateNode(node.id));
            }
            if !node.size.width.is_finite()
                || !node.size.height.is_finite()
                || node.size.width <= 0.0
                || node.size.height <= 0.0
            {
                return Err(LayoutError::InvalidSize(node.id));
            }
            match node.pin {
                Pin::Prior { position, weight }
                    if !finite_point(position) || !weight.is_finite() || weight < 0.0 =>
                {
                    return Err(LayoutError::InvalidConfig("invalid prior pin"));
                }
                Pin::Fixed(position) if !finite_point(position) => {
                    return Err(LayoutError::InvalidConfig("invalid fixed pin"));
                }
                _ => {}
            }
        }

        let index: BTreeMap<_, _> = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id, index))
            .collect();
        let mut edges = input.edges.clone();
        edges.sort_by_key(|edge| edge.id);
        let mut edge_ids = BTreeSet::new();
        for edge in &edges {
            if !edge_ids.insert(edge.id) {
                return Err(LayoutError::DuplicateEdge(edge.id));
            }
            for endpoint in [edge.source, edge.target] {
                if !index.contains_key(&endpoint) {
                    return Err(LayoutError::MissingEndpoint {
                        edge: edge.id,
                        node: endpoint,
                    });
                }
            }
            if edge.source == edge.target
                || !edge.ideal_length.is_finite()
                || edge.ideal_length <= 0.0
                || !edge.weight.is_finite()
                || edge.weight <= 0.0
                || matches!(edge.kind, EdgeKind::Directed { target_delta } if !target_delta.is_finite())
            {
                return Err(LayoutError::InvalidEdge(edge.id));
            }
        }

        let constraints = input.constraints.clone();
        for constraint in &constraints {
            match constraint {
                AxisConstraint::Position {
                    node,
                    coordinate,
                    weight,
                    ..
                } => {
                    validate_constraint_nodes(&index, &[*node])?;
                    validate_constraint_scalars(&[*coordinate], *weight)?;
                }
                AxisConstraint::Offset {
                    source,
                    target,
                    delta,
                    weight,
                    ..
                } => {
                    validate_constraint_nodes(&index, &[*source, *target])?;
                    validate_constraint_scalars(&[*delta], *weight)?;
                    if source == target {
                        return Err(LayoutError::InvalidConfig(
                            "axis constraint cannot relate a node to itself",
                        ));
                    }
                }
                AxisConstraint::Separation {
                    before,
                    after,
                    minimum,
                    weight,
                    ..
                } => {
                    validate_constraint_nodes(&index, &[*before, *after])?;
                    validate_constraint_scalars(&[*minimum], *weight)?;
                    if before == after {
                        return Err(LayoutError::InvalidConfig(
                            "axis constraint cannot relate a node to itself",
                        ));
                    }
                    if *minimum < 0.0 {
                        return Err(LayoutError::InvalidConfig(
                            "axis separation cannot be negative",
                        ));
                    }
                }
            }
        }

        let mut free_slot = vec![None; nodes.len()];
        let mut fixed = vec![None; nodes.len()];
        let mut slot = 0;
        for (index, node) in nodes.iter().enumerate() {
            if let Pin::Fixed(point) = node.pin {
                fixed[index] = Some(point);
            } else {
                free_slot[index] = Some(slot);
                slot += 1;
            }
        }
        let repulsion_scale = if edges.is_empty() {
            10_000.0
        } else {
            let mean = edges.iter().map(|edge| edge.ideal_length).sum::<f64>() / edges.len() as f64;
            mean * mean
        };
        Ok(Self {
            nodes,
            edges,
            constraints,
            index,
            free_slot,
            fixed,
            config: input.config.clone(),
            repulsion_scale,
        })
    }

    pub(crate) fn parameter_count(&self) -> usize {
        self.free_slot.iter().flatten().count() * 2
    }

    pub(crate) fn positions(&self, params: &DVector<f64>) -> Vec<Point> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, _)| {
                if let Some(point) = self.fixed[index] {
                    point
                } else {
                    let slot = self.free_slot[index].expect("free node has slot");
                    Point::new(params[2 * slot], params[2 * slot + 1])
                }
            })
            .collect()
    }

    pub(crate) fn params_from_positions(&self, positions: &[Point]) -> DVector<f64> {
        let mut params = DVector::zeros(self.parameter_count());
        for (index, point) in positions.iter().enumerate() {
            if let Some(slot) = self.free_slot[index] {
                params[2 * slot] = point.x;
                params[2 * slot + 1] = point.y;
            }
        }
        params
    }

    pub(crate) fn breakdown(&self, params: &DVector<f64>) -> EnergyBreakdown {
        let positions = self.positions(params);
        let mut out = EnergyBreakdown::default();
        for edge in &self.edges {
            let source = positions[self.index[&edge.source]];
            let target = positions[self.index[&edge.target]];
            let distance = source.distance(target).max(1e-7);
            let residual = distance - edge.ideal_length;
            out.stress += self.config.stress_weight * edge.weight * residual * residual;
            if let EdgeKind::Directed { target_delta } = edge.kind {
                let residual = target.y - source.y - target_delta;
                out.hierarchy += self.config.hierarchy_weight * edge.weight * residual * residual;
            }
        }

        out.total = self.evaluate(params, None);
        out
    }

    fn evaluate(&self, params: &DVector<f64>, mut gradient: Option<&mut DVector<f64>>) -> f64 {
        let positions = self.positions(params);
        if let Some(g) = gradient.as_deref_mut() {
            g.fill(0.0);
        }
        let mut cost = 0.0;

        for edge in &self.edges {
            let source_index = self.index[&edge.source];
            let target_index = self.index[&edge.target];
            let source = positions[source_index];
            let target = positions[target_index];
            let dx = target.x - source.x;
            let dy = target.y - source.y;
            let distance = dx.hypot(dy).max(1e-7);
            let residual = distance - edge.ideal_length;
            let weight = self.config.stress_weight * edge.weight;
            cost += weight * residual * residual;
            let scale = 2.0 * weight * residual / distance;
            add_pair_gradient(
                &self.free_slot,
                &mut gradient,
                source_index,
                target_index,
                scale * dx,
                scale * dy,
            );

            if let EdgeKind::Directed { target_delta } = edge.kind {
                let residual = dy - target_delta;
                let weight = self.config.hierarchy_weight * edge.weight;
                cost += weight * residual * residual;
                add_pair_gradient(
                    &self.free_slot,
                    &mut gradient,
                    source_index,
                    target_index,
                    0.0,
                    2.0 * weight * residual,
                );
            }
        }

        for constraint in &self.constraints {
            apply_constraint_energy(
                constraint,
                &self.index,
                &self.free_slot,
                &positions,
                &mut cost,
                &mut gradient,
            );
        }

        // Crossing edges repel at their intersection basin. This is a local,
        // differentiable surrogate within each fixed crossing topology; exact
        // crossing counts remain a separately reported metric.
        for left in 0..self.edges.len() {
            let a = &self.edges[left];
            for b in &self.edges[left + 1..] {
                if a.source == b.source
                    || a.source == b.target
                    || a.target == b.source
                    || a.target == b.target
                {
                    continue;
                }
                let ai = self.index[&a.source];
                let aj = self.index[&a.target];
                let bi = self.index[&b.source];
                let bj = self.index[&b.target];
                if segments_intersect(positions[ai], positions[aj], positions[bi], positions[bj])
                    .is_none()
                {
                    continue;
                }
                let midpoint_a = Point::new(
                    (positions[ai].x + positions[aj].x) * 0.5,
                    (positions[ai].y + positions[aj].y) * 0.5,
                );
                let midpoint_b = Point::new(
                    (positions[bi].x + positions[bj].x) * 0.5,
                    (positions[bi].y + positions[bj].y) * 0.5,
                );
                let mut dx = midpoint_b.x - midpoint_a.x;
                let mut dy = midpoint_b.y - midpoint_a.y;
                if dx.hypot(dy) < 1e-7 {
                    dx = -(positions[aj].y - positions[ai].y);
                    dy = positions[aj].x - positions[ai].x;
                    let length = dx.hypot(dy).max(1e-7);
                    dx /= length;
                    dy /= length;
                }
                let distance2 = dx * dx + dy * dy + 64.0;
                let strength = self.config.crossing_weight * self.repulsion_scale * 4.0;
                cost += strength / distance2;
                let gx = strength * dx / (distance2 * distance2);
                let gy = strength * dy / (distance2 * distance2);
                add_node_gradient(&self.free_slot, &mut gradient, ai, gx, gy);
                add_node_gradient(&self.free_slot, &mut gradient, aj, gx, gy);
                add_node_gradient(&self.free_slot, &mut gradient, bi, -gx, -gy);
                add_node_gradient(&self.free_slot, &mut gradient, bj, -gx, -gy);
            }
        }

        for left in 0..positions.len() {
            for right in left + 1..positions.len() {
                let dx = positions[right].x - positions[left].x;
                let dy = positions[right].y - positions[left].y;
                let distance2 = dx * dx + dy * dy + 64.0;
                let repulsion = self.config.repulsion_weight * self.repulsion_scale / distance2;
                cost += repulsion;
                let scale = -2.0 * self.config.repulsion_weight * self.repulsion_scale
                    / (distance2 * distance2);
                add_pair_gradient(
                    &self.free_slot,
                    &mut gradient,
                    left,
                    right,
                    scale * dx,
                    scale * dy,
                );

                let gap_x = (self.nodes[left].size.width + self.nodes[right].size.width) * 0.5
                    + self.config.clearance;
                let gap_y = (self.nodes[left].size.height + self.nodes[right].size.height) * 0.5
                    + self.config.clearance;
                let overlap_x = gap_x - dx.abs();
                let overlap_y = gap_y - dy.abs();
                if overlap_x > 0.0 && overlap_y > 0.0 {
                    let (gx, gy, depth) = if overlap_x <= overlap_y {
                        (-dx.signum_or(left, right), 0.0, overlap_x)
                    } else {
                        (0.0, -dy.signum_or(left, right), overlap_y)
                    };
                    let weight = self.config.overlap_weight;
                    cost += weight * depth * depth;
                    add_pair_gradient(
                        &self.free_slot,
                        &mut gradient,
                        left,
                        right,
                        2.0 * weight * depth * gx,
                        2.0 * weight * depth * gy,
                    );
                }
            }
        }

        for (index, node) in self.nodes.iter().enumerate() {
            if let Pin::Prior { position, weight } = node.pin {
                let dx = positions[index].x - position.x;
                let dy = positions[index].y - position.y;
                cost += weight * (dx * dx + dy * dy);
                if let (Some(slot), Some(g)) = (self.free_slot[index], gradient.as_deref_mut()) {
                    g[2 * slot] += 2.0 * weight * dx;
                    g[2 * slot + 1] += 2.0 * weight * dy;
                }
            }
        }

        // Remove translational drift without meaningfully affecting geometry.
        if !positions.is_empty() {
            let cx = positions.iter().map(|point| point.x).sum::<f64>() / positions.len() as f64;
            let cy = positions.iter().map(|point| point.y).sum::<f64>() / positions.len() as f64;
            let weight = 1e-5;
            cost += weight * (cx * cx + cy * cy);
            if let Some(g) = gradient {
                for (index, slot) in self
                    .free_slot
                    .iter()
                    .enumerate()
                    .filter_map(|(index, slot)| slot.map(|slot| (index, slot)))
                {
                    g[2 * slot] += 2.0 * weight * cx / positions.len() as f64;
                    g[2 * slot + 1] += 2.0 * weight * cy / positions.len() as f64;
                    // A weak gravitational well bounds disconnected components;
                    // pair repulsion alone has no finite minimum for them.
                    let compactness = 1e-5;
                    cost += compactness
                        * (positions[index].x * positions[index].x
                            + positions[index].y * positions[index].y);
                    g[2 * slot] += 2.0 * compactness * positions[index].x;
                    g[2 * slot + 1] += 2.0 * compactness * positions[index].y;
                }
            } else {
                let compactness = 1e-5;
                cost += positions
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| self.free_slot[*index].is_some())
                    .map(|(_, point)| compactness * (point.x * point.x + point.y * point.y))
                    .sum::<f64>();
            }
        }
        cost
    }
}

fn validate_constraint_nodes(
    index: &BTreeMap<NodeId, usize>,
    nodes: &[NodeId],
) -> Result<(), LayoutError> {
    if nodes.iter().any(|node| !index.contains_key(node)) {
        return Err(LayoutError::InvalidConfig(
            "axis constraint references a missing node",
        ));
    }
    Ok(())
}

fn validate_constraint_scalars(values: &[f64], weight: f64) -> Result<(), LayoutError> {
    if values.iter().any(|value| !value.is_finite()) || !weight.is_finite() || weight <= 0.0 {
        return Err(LayoutError::InvalidConfig("invalid axis constraint"));
    }
    Ok(())
}

fn axis_value(point: Point, axis: Axis) -> f64 {
    match axis {
        Axis::Horizontal => point.x,
        Axis::Vertical => point.y,
    }
}

fn add_axis_gradient(
    slots: &[Option<usize>],
    gradient: &mut Option<&mut DVector<f64>>,
    node: usize,
    axis: Axis,
    value: f64,
) {
    let Some(slot) = slots[node] else {
        return;
    };
    let Some(gradient) = gradient.as_deref_mut() else {
        return;
    };
    gradient[2 * slot + usize::from(axis == Axis::Vertical)] += value;
}

fn apply_constraint_energy(
    constraint: &AxisConstraint,
    index: &BTreeMap<NodeId, usize>,
    slots: &[Option<usize>],
    positions: &[Point],
    cost: &mut f64,
    gradient: &mut Option<&mut DVector<f64>>,
) {
    match *constraint {
        AxisConstraint::Position {
            node,
            axis,
            coordinate,
            weight,
        } => {
            let node = index[&node];
            let residual = axis_value(positions[node], axis) - coordinate;
            *cost += weight * residual * residual;
            add_axis_gradient(slots, gradient, node, axis, 2.0 * weight * residual);
        }
        AxisConstraint::Offset {
            source,
            target,
            axis,
            delta,
            weight,
        } => {
            let source = index[&source];
            let target = index[&target];
            let residual =
                axis_value(positions[target], axis) - axis_value(positions[source], axis) - delta;
            *cost += weight * residual * residual;
            add_axis_gradient(slots, gradient, source, axis, -2.0 * weight * residual);
            add_axis_gradient(slots, gradient, target, axis, 2.0 * weight * residual);
        }
        AxisConstraint::Separation {
            before,
            after,
            axis,
            minimum,
            weight,
        } => {
            let before = index[&before];
            let after = index[&after];
            let actual = axis_value(positions[after], axis) - axis_value(positions[before], axis);
            let violation = minimum - actual;
            if violation > 0.0 {
                *cost += weight * violation * violation;
                add_axis_gradient(slots, gradient, before, axis, 2.0 * weight * violation);
                add_axis_gradient(slots, gradient, after, axis, -2.0 * weight * violation);
            }
        }
    }
}

impl CostFunction for CompiledProblem {
    type Param = DVector<f64>;
    type Output = f64;

    fn cost(&self, params: &Self::Param) -> Result<Self::Output, ArgminError> {
        Ok(self.evaluate(params, None))
    }
}

impl Gradient for CompiledProblem {
    type Param = DVector<f64>;
    type Gradient = DVector<f64>;

    fn gradient(&self, params: &Self::Param) -> Result<Self::Gradient, ArgminError> {
        let mut gradient = DVector::zeros(params.len());
        self.evaluate(params, Some(&mut gradient));
        Ok(gradient)
    }
}

fn add_pair_gradient(
    slots: &[Option<usize>],
    gradient: &mut Option<&mut DVector<f64>>,
    source: usize,
    target: usize,
    target_x: f64,
    target_y: f64,
) {
    let Some(g) = gradient.as_deref_mut() else {
        return;
    };
    if let Some(slot) = slots[source] {
        g[2 * slot] -= target_x;
        g[2 * slot + 1] -= target_y;
    }
    if let Some(slot) = slots[target] {
        g[2 * slot] += target_x;
        g[2 * slot + 1] += target_y;
    }
}

fn add_node_gradient(
    slots: &[Option<usize>],
    gradient: &mut Option<&mut DVector<f64>>,
    node: usize,
    x: f64,
    y: f64,
) {
    let Some(slot) = slots[node] else {
        return;
    };
    let Some(g) = gradient.as_deref_mut() else {
        return;
    };
    g[2 * slot] += x;
    g[2 * slot + 1] += y;
}

trait StableSign {
    fn signum_or(self, left: usize, right: usize) -> f64;
}

impl StableSign for f64 {
    fn signum_or(self, left: usize, right: usize) -> f64 {
        if self > 0.0 {
            1.0
        } else if self < 0.0 {
            -1.0
        } else if left < right {
            1.0
        } else {
            -1.0
        }
    }
}

fn finite_point(point: Point) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn validate_config(config: &LayoutConfig) -> Result<(), LayoutError> {
    let finite_nonnegative = [
        config.gradient_tolerance,
        config.hierarchy_weight,
        config.stress_weight,
        config.repulsion_weight,
        config.overlap_weight,
        config.crossing_weight,
        config.clearance,
        config.route_clearance,
        config.component_gap,
    ]
    .into_iter()
    .all(|value| value.is_finite() && value >= 0.0);
    if !finite_nonnegative {
        return Err(LayoutError::InvalidConfig(
            "weights and distances must be finite and nonnegative",
        ));
    }
    if config.max_iterations == 0 {
        return Err(LayoutError::InvalidConfig(
            "max_iterations must be positive",
        ));
    }
    if config.history_size == 0 {
        return Err(LayoutError::InvalidConfig("history_size must be positive"));
    }
    Ok(())
}
