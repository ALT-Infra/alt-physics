//! Deterministic constrained-energy graph layout.
//!
//! This crate deliberately knows nothing about application roles or rendering
//! frameworks. Callers translate their semantics into geometric constraints.

mod energy;
mod error;
mod geometry;
mod initialize;
mod metrics;
mod model;
mod routing;
mod solver;

pub use error::LayoutError;
pub use model::{
    Edge, EdgeId, EdgeKind, LayoutConfig, LayoutInput, LayoutMetrics, LayoutOutput, Node, NodeId,
    NodePlacement, Pin, Point, Port, Route, Side, Size, SolverDiagnostics,
};
pub use solver::layout;
