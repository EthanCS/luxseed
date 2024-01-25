use crate::node::NodeHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    NodeEdge {
        input_node: NodeHandle,
        output_node: NodeHandle,
    },
    ResourceEdge {
        input_node: NodeHandle,
        input_slot: usize,
        output_node: NodeHandle,
        output_slot: usize,
    },
}

impl Edge {
    pub fn get_input_node(&self) -> NodeHandle {
        match self {
            Edge::NodeEdge { input_node: input, .. } => *input,
            Edge::ResourceEdge { input_node: input, .. } => *input,
        }
    }

    pub fn get_output_node(&self) -> NodeHandle {
        match self {
            Edge::NodeEdge { output_node: output, .. } => *output,
            Edge::ResourceEdge { output_node: output, .. } => *output,
        }
    }
}
