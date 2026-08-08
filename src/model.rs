use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type NodeId = u64;
pub type EdgeId = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub(crate) fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Port {
    /// Choose the rectangle boundary point from the final route direction.
    #[default]
    Free,
    /// Choose any point on this side.
    Side(Side),
    /// Fix a normalized offset along a side. `-1` is left/top and `1` is right/bottom.
    Fixed { side: Side, offset: f64 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Pin {
    #[default]
    Free,
    Prior {
        position: Point,
        weight: f64,
    },
    Fixed(Point),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node {
    pub id: NodeId,
    pub size: Size,
    #[cfg_attr(feature = "serde", serde(default))]
    pub pin: Pin,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EdgeKind {
    /// Symmetric spring with no directional preference.
    Association,
    /// Spring plus a target displacement along the vertical flow axis.
    Directed { target_delta: f64 },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
    pub ideal_length: f64,
    pub weight: f64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub source_port: Port,
    #[cfg_attr(feature = "serde", serde(default))]
    pub target_port: Port,
}

/// A geometric relationship that affects placement without manufacturing a
/// rendered graph edge.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisConstraint {
    /// Prefer one coordinate for a node.
    Position {
        node: NodeId,
        axis: Axis,
        coordinate: f64,
        weight: f64,
    },
    /// Prefer `target - source == delta` along one axis.
    Offset {
        source: NodeId,
        target: NodeId,
        axis: Axis,
        delta: f64,
        weight: f64,
    },
    /// Require `after - before >= minimum` along one axis. The inequality is
    /// both an energy term and an exact post-optimization projection.
    Separation {
        before: NodeId,
        after: NodeId,
        axis: Axis,
        minimum: f64,
        weight: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutConfig {
    pub seed: u64,
    pub max_iterations: u64,
    pub history_size: usize,
    pub gradient_tolerance: f64,
    pub hierarchy_weight: f64,
    pub stress_weight: f64,
    pub repulsion_weight: f64,
    pub overlap_weight: f64,
    pub crossing_weight: f64,
    pub clearance: f64,
    pub route_clearance: f64,
    pub component_gap: f64,
    pub projection_passes: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            seed: 0xA17_2026,
            max_iterations: 280,
            history_size: 12,
            gradient_tolerance: 1e-6,
            hierarchy_weight: 2.4,
            stress_weight: 1.0,
            repulsion_weight: 0.32,
            overlap_weight: 18.0,
            crossing_weight: 0.18,
            clearance: 28.0,
            route_clearance: 12.0,
            component_gap: 120.0,
            projection_passes: 24,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutInput {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub constraints: Vec<AxisConstraint>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub config: LayoutConfig,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodePlacement {
    /// Center position in graph space.
    pub center: Point,
    pub size: Size,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Route {
    pub edge: EdgeId,
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutMetrics {
    pub energy: f64,
    pub stress: f64,
    pub hierarchy_error: f64,
    pub overlaps: usize,
    pub crossings: usize,
    pub minimum_crossing_angle_degrees: Option<f64>,
    pub total_edge_length: f64,
    pub bends: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SolverDiagnostics {
    pub iterations: u64,
    pub termination: String,
    pub projected_pairs: usize,
    pub routed_obstacles: usize,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutOutput {
    pub placements: BTreeMap<NodeId, NodePlacement>,
    pub routes: Vec<Route>,
    pub metrics: LayoutMetrics,
    pub diagnostics: SolverDiagnostics,
}
