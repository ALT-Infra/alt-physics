use crate::{EdgeId, NodeId};

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("node ids must be unique; duplicate id {0}")]
    DuplicateNode(NodeId),
    #[error("edge ids must be unique; duplicate id {0}")]
    DuplicateEdge(EdgeId),
    #[error("edge {edge} refers to missing node {node}")]
    MissingEndpoint { edge: EdgeId, node: NodeId },
    #[error("node {0} has invalid dimensions")]
    InvalidSize(NodeId),
    #[error("edge {0} has invalid geometry or weight")]
    InvalidEdge(EdgeId),
    #[error("layout configuration is invalid: {0}")]
    InvalidConfig(&'static str),
    #[error("numerical optimization failed: {0}")]
    Optimization(String),
}
