use crate::node::NodeHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    NodeEdge {
        input_node_handle: NodeHandle,
        output_node_handle: NodeHandle,
    },
    ResourceEdge {
        input_node_handle: NodeHandle,
        input_slot_index: usize,
        output_node_handle: NodeHandle,
        output_slot_index: usize,
    },
}

impl Edge {
    pub fn get_input_node(&self) -> NodeHandle {
        match self {
            Edge::NodeEdge { input_node_handle: input, .. } => *input,
            Edge::ResourceEdge { input_node_handle: input, .. } => *input,
        }
    }

    pub fn get_output_node(&self) -> NodeHandle {
        match self {
            Edge::NodeEdge { output_node_handle: output, .. } => *output,
            Edge::ResourceEdge { output_node_handle: output, .. } => *output,
        }
    }
}
