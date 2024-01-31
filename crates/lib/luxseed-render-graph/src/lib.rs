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
    #[error("Node {0:?} not found")]
    InvalidNode(NodeIdentifier),
    #[error("Node {0:?} has no input slot {1:?}")]
    InvalidNodeInputSlot(NodeIdentifier, ResourceSlotIdentifier),
    #[error("Node {0:?} has no output slot {1:?}")]
    InvalidNodeOutputSlot(NodeIdentifier, ResourceSlotIdentifier),
    #[error("Edge {0:?} already exists")]
    EdgeAlreadyExists(Edge),
    #[error("Edge {0:?} does not exist")]
    EdgeDoesNotExist(Edge),
    #[error("Node {0:?} update error")]
    NodeOnUpdateError(NodeIdentifier),
    #[error("Node {0:?} render error")]
    NodeOnRenderError(NodeIdentifier),
    #[error("Unknown render graph error")]
    Unknown,
}
