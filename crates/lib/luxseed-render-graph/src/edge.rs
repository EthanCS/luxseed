use crate::node::NodeHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    NodeEdge { input: NodeHandle, output: NodeHandle },
    ResourceEdge { input: NodeHandle, output: NodeHandle },
}

impl Edge {
    pub fn get_input_node(&self) -> NodeHandle {
        match self {
            Edge::NodeEdge { input, .. } => *input,
            Edge::ResourceEdge { input, .. } => *input,
        }
    }

    pub fn get_output_node(&self) -> NodeHandle {
        match self {
            Edge::NodeEdge { output, .. } => *output,
            Edge::ResourceEdge { output, .. } => *output,
        }
    }
}
