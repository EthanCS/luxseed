mod context;
mod edge;
mod graph;
mod node;
mod resource;

use crate::graph::RenderGraph;
use edge::Edge;
use luxseed_render_backend::RenderBackend;
use node::NodeIdentifier;
use resource::ResourceSlotIdentifier;
use thiserror::Error;

pub struct RenderGraphSystem;

impl RenderGraphSystem {
    pub fn run(backend: &mut Box<dyn RenderBackend>, render_graph: &RenderGraph) {
        Self::run_render_graph(backend, render_graph);
    }

    fn run_render_graph(backend: &mut Box<dyn RenderBackend>, render_graph: &RenderGraph) {}
}

#[derive(Error, Debug)]
pub enum RenderGraphError {
    #[error("node {0:?} not found")]
    InvalidNode(NodeIdentifier),
    #[error("node {0:?} has no input slot {1:?}")]
    InvalidNodeInputSlot(NodeIdentifier, ResourceSlotIdentifier),
    #[error("node {0:?} has no output slot {1:?}")]
    InvalidNodeOutputSlot(NodeIdentifier, ResourceSlotIdentifier),
    #[error("node {0:?} slot {1:?} and Node {2:?} Slot {3:?} type mismatch")]
    MismatchNodeSlotResourceType(
        NodeIdentifier,
        ResourceSlotIdentifier,
        NodeIdentifier,
        ResourceSlotIdentifier,
    ),
    #[error("node {0:?} input slot {1:?} already connected by node {2:?}")]
    NodeInputSlotAlreadyConnected(NodeIdentifier, ResourceSlotIdentifier, NodeIdentifier),
    #[error("edge {0:?} already exists")]
    EdgeAlreadyExists(Edge),
    #[error("edge {0:?} does not exist")]
    EdgeDoesNotExist(Edge),
    #[error("node {0:?} update error")]
    NodeOnUpdateError(NodeIdentifier),
    #[error("node {0:?} render error")]
    NodeOnRenderError(NodeIdentifier),
    #[error("unknown render graph error")]
    Unknown,
}
