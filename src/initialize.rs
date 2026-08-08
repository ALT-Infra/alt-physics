use nalgebra::{linalg::SymmetricEigen, DMatrix, DVector};
use petgraph::unionfind::UnionFind;

use crate::{energy::CompiledProblem, EdgeKind, Pin, Point};

pub(crate) fn initialize(problem: &CompiledProblem) -> Vec<Point> {
    let count = problem.nodes.len();
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![match problem.nodes[0].pin {
            Pin::Fixed(point)
            | Pin::Prior {
                position: point, ..
            } => point,
            Pin::Free => Point::default(),
        }];
    }

    let mut laplacian = DMatrix::<f64>::zeros(count, count);
    for edge in &problem.edges {
        let i = problem.index[&edge.source];
        let j = problem.index[&edge.target];
        let weight = edge.weight.max(1e-6);
        laplacian[(i, i)] += weight;
        laplacian[(j, j)] += weight;
        laplacian[(i, j)] -= weight;
        laplacian[(j, i)] -= weight;
    }

    let eigen = SymmetricEigen::new(laplacian.clone());
    let mut order: Vec<_> = (0..count).collect();
    order.sort_by(|&a, &b| {
        eigen.eigenvalues[a]
            .total_cmp(&eigen.eigenvalues[b])
            .then_with(|| a.cmp(&b))
    });
    let axis = order.get(1).copied().unwrap_or(order[0]);
    let scale = problem
        .edges
        .iter()
        .map(|edge| edge.ideal_length)
        .sum::<f64>()
        / problem.edges.len().max(1) as f64
        * (count as f64).sqrt();

    let mut y_matrix = laplacian;
    let mut y_rhs = DVector::<f64>::zeros(count);
    for edge in &problem.edges {
        let EdgeKind::Directed { target_delta } = edge.kind else {
            continue;
        };
        let i = problem.index[&edge.source];
        let j = problem.index[&edge.target];
        y_rhs[i] -= edge.weight * target_delta;
        y_rhs[j] += edge.weight * target_delta;
    }
    for i in 0..count {
        let baseline = deterministic_unit(problem.nodes[i].id ^ problem.config.seed);
        y_matrix[(i, i)] += 1e-4;
        y_rhs[i] += 1e-4 * baseline * scale;
        match problem.nodes[i].pin {
            Pin::Fixed(point) => {
                y_matrix[(i, i)] += 1e7;
                y_rhs[i] += 1e7 * point.y;
            }
            Pin::Prior { position, weight } => {
                y_matrix[(i, i)] += weight;
                y_rhs[i] += weight * position.y;
            }
            Pin::Free => {}
        }
    }
    let y = y_matrix
        .lu()
        .solve(&y_rhs)
        .unwrap_or_else(|| DVector::zeros(count));

    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let jitter = deterministic_unit(problem.nodes[i].id.rotate_left(17) ^ problem.config.seed);
        let spectral = eigen.eigenvectors[(i, axis)] * scale;
        let mut point = Point::new(spectral + jitter * scale * 0.02, y[i]);
        match problem.nodes[i].pin {
            Pin::Fixed(fixed) => point = fixed,
            Pin::Prior { position, weight } if weight >= 1.0 => {
                let blend = (weight / (weight + 1.0)).clamp(0.0, 0.98);
                point.x = point.x * (1.0 - blend) + position.x * blend;
                point.y = point.y * (1.0 - blend) + position.y * blend;
            }
            _ => {}
        }
        positions.push(point);
    }
    separate_components(problem, &mut positions);
    positions
}

fn separate_components(problem: &CompiledProblem, positions: &mut [Point]) {
    if positions.len() < 2
        || problem
            .nodes
            .iter()
            .any(|node| matches!(node.pin, Pin::Fixed(_)))
    {
        return;
    }
    let mut union = UnionFind::new(positions.len());
    for edge in &problem.edges {
        union.union(problem.index[&edge.source], problem.index[&edge.target]);
    }
    let mut components = std::collections::BTreeMap::<usize, Vec<usize>>::new();
    for index in 0..positions.len() {
        components.entry(union.find(index)).or_default().push(index);
    }
    if components.len() <= 1 {
        return;
    }
    let mut groups: Vec<_> = components.into_values().collect();
    groups.sort_by_key(|group| group.iter().map(|&index| problem.nodes[index].id).min());
    let mut cursor = 0.0;
    for group in groups {
        let left = group
            .iter()
            .map(|&index| positions[index].x - problem.nodes[index].size.width * 0.5)
            .fold(f64::INFINITY, f64::min);
        let right = group
            .iter()
            .map(|&index| positions[index].x + problem.nodes[index].size.width * 0.5)
            .fold(f64::NEG_INFINITY, f64::max);
        let shift = cursor - left;
        for index in group {
            positions[index].x += shift;
        }
        cursor += right - left + problem.config.component_gap;
    }
    let center = positions.iter().map(|point| point.x).sum::<f64>() / positions.len() as f64;
    for point in positions {
        point.x -= center;
    }
}

fn deterministic_unit(mut value: u64) -> f64 {
    value = value.wrapping_add(0x9E3779B97F4A7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
    value ^= value >> 31;
    (value as f64 / u64::MAX as f64) * 2.0 - 1.0
}
