use crate::{
    edge::Edge,
    node::{Node, NodeHandle, NodeIdentifier},
    resource::{ResourceSlot, ResourceSlotIdentifier},
    RenderGraphError,
};
use std::{borrow::Cow, collections::HashMap};

const MAX_RESOURCES_COUNT: usize = 1024;
const MAX_NODES_COUNT: usize = 1024;

#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<NodeHandle, Node>,
    node_names: HashMap<Cow<'static, str>, NodeHandle>,
}

impl RenderGraph {
    pub fn update(&mut self) -> Result<(), RenderGraphError> {
        for node in self.nodes.values_mut() {
            if let Some(on_update) = node.on_update.take() {
                on_update().map_err(|_| RenderGraphError::NodeOnUpdateError(node.handle.into()))?;
            }
        }
        Ok(())
    }

    pub fn get_node_handle(
        &self,
        identifier: impl Into<NodeIdentifier>,
    ) -> Result<NodeHandle, RenderGraphError> {
        let identifier = identifier.into();
        match identifier {
            NodeIdentifier::Name(ref name) => {
                self.node_names.get(name).copied().ok_or(RenderGraphError::InvalidNode(identifier))
            }
            NodeIdentifier::Handle(handle) => Ok(handle),
        }
    }

    pub fn get_node(
        &self,
        identifier: impl Into<NodeIdentifier>,
    ) -> Result<&Node, RenderGraphError> {
        let identifier = identifier.into();
        let node_handle = self.get_node_handle(&identifier)?;
        self.nodes.get(&node_handle).ok_or(RenderGraphError::InvalidNode(identifier))
    }

    pub fn get_node_mut(
        &mut self,
        identifier: impl Into<NodeIdentifier>,
    ) -> Result<&mut Node, RenderGraphError> {
        let identifier = identifier.into();
        let node_handle = self.get_node_handle(&identifier)?;
        self.nodes.get_mut(&node_handle).ok_or(RenderGraphError::InvalidNode(identifier))
    }

    pub fn add_node(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        input_resources: &[ResourceSlot],
        output_resources: &[ResourceSlot],
    ) -> NodeHandle {
        let handle = NodeHandle::new();
        let name = name.into();
        let mut node = Node::new(handle, input_resources, output_resources);
        node.name = Some(name.clone());
        self.nodes.insert(handle, node);
        self.node_names.insert(name, handle);
        handle
    }

    pub fn remove_node(
        &mut self,
        name: impl Into<Cow<'static, str>>,
    ) -> Result<(), RenderGraphError> {
        todo!()
    }

    /// Return true if the graph has this edge.
    pub fn has_edge(&self, edge: &Edge) -> bool {
        let output_node = self.get_node(edge.get_output_node());
        let input_node = self.get_node(edge.get_input_node());
        if let Ok(output_node) = output_node {
            if output_node.output_edges().contains(edge) {
                if let Ok(input_node) = input_node {
                    if input_node.input_edges().contains(edge) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn validate_edge(&self, edge: &Edge, should_exist: bool) -> Result<(), RenderGraphError> {
        if should_exist && !self.has_edge(edge) {
            return Err(RenderGraphError::EdgeDoesNotExist(edge.clone()));
        } else if !should_exist && self.has_edge(edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge.clone()));
        }

        match *edge {
            Edge::ResourceEdge {
                output_node_handle,
                output_slot_index,
                input_node_handle,
                input_slot_index,
            } => {
                let output_node = self.get_node(output_node_handle)?;
                let output_slot = output_node.output_slots().get_slot(output_slot_index).ok_or(
                    RenderGraphError::InvalidNodeOutputSlot(
                        output_node_handle.into(),
                        output_slot_index.into(),
                    ),
                )?;

                let input_node = self.get_node(input_node_handle)?;
                let input_slot = input_node.input_slots().get_slot(input_slot_index).ok_or(
                    RenderGraphError::InvalidNodeInputSlot(
                        input_node_handle.into(),
                        input_slot_index.into(),
                    ),
                )?;

                if output_slot.resource_type != input_slot.resource_type {
                    return Err(RenderGraphError::MismatchNodeSlotResourceType(
                        output_node_handle.into(),
                        output_slot_index.into(),
                        input_node_handle.into(),
                        input_slot_index.into(),
                    ));
                }

                // Check if the input slot is already connected to another output slot.
                if let Some(Edge::ResourceEdge {
                    output_node_handle: current_output_node, ..
                }) = input_node.input_edges().iter().find(|e| {
                    if let Edge::ResourceEdge { input_slot_index: current_input_index, .. } = e {
                        input_slot_index == *current_input_index
                    } else {
                        false
                    }
                }) {
                    if !should_exist {
                        return Err(RenderGraphError::NodeInputSlotAlreadyConnected(
                            input_node_handle.into(),
                            input_slot_index.into(),
                            (*current_output_node).into(),
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn try_add_node_edge(
        &mut self,
        output: impl Into<NodeIdentifier>,
        input: impl Into<NodeIdentifier>,
    ) -> Result<(), RenderGraphError> {
        let output = output.into();
        let output_node_handle = self.get_node_handle(&output)?;
        let input = input.into();
        let input_node_handle = self.get_node_handle(&input)?;

        let new_edge = Edge::NodeEdge { output_node_handle, input_node_handle };
        self.validate_edge(&new_edge, false)?;

        {
            let output_node = self.get_node_mut(&output)?;
            output_node.add_output_edge(new_edge)?;
        }
        let input_node = self.get_node_mut(&input)?;
        input_node.add_input_edge(new_edge)?;

        Ok(())
    }

    pub fn remove_node_edge(
        &mut self,
        output: impl Into<NodeIdentifier>,
        input: impl Into<NodeIdentifier>,
    ) -> Result<(), RenderGraphError> {
        todo!()
    }

    pub fn try_add_resource_edge(
        &mut self,
        output_node: impl Into<NodeIdentifier>,
        output_slot: impl Into<ResourceSlotIdentifier>,
        input_node: impl Into<NodeIdentifier>,
        input_slot: impl Into<ResourceSlotIdentifier>,
    ) -> Result<(), RenderGraphError> {
        let output_node = output_node.into();
        let output_slot = output_slot.into();
        let output_node_handle = self.get_node_handle(&output_node)?;
        let output_slot_index = self
            .get_node(&output_node)?
            .output_slots()
            .get_slot_index(&output_slot)
            .ok_or(RenderGraphError::InvalidNodeOutputSlot(output_node, output_slot))?;

        let input_slot = input_slot.into();
        let input_node = input_node.into();
        let input_node_handle = self.get_node_handle(&input_node)?;
        let input_slot_index = self
            .get_node(&input_node)?
            .input_slots()
            .get_slot_index(&input_slot)
            .ok_or(RenderGraphError::InvalidNodeInputSlot(input_node, input_slot))?;

        let new_edge = Edge::ResourceEdge {
            output_node_handle,
            output_slot_index,
            input_node_handle,
            input_slot_index,
        };
        self.validate_edge(&new_edge, false)?;

        {
            let output_node = self.get_node_mut(output_node_handle)?;
            output_node.add_output_edge(new_edge)?;
        }
        let input_node = self.get_node_mut(input_node_handle)?;
        input_node.add_input_edge(new_edge)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::resource::ResourceSlot;

    use super::RenderGraph;

    #[derive(Copy, Clone)]
    struct WorldData {
        pub value: i32,
    }

    impl WorldData {
        fn new(value: i32) -> Self {
            Self { value }
        }

        fn update(&mut self) {
            self.value += 1;
        }
    }

    #[test]
    fn single_node_update_logic() {
        let mut data = WorldData::new(0);

        let mut rg = RenderGraph::default();
        let handle = rg.add_node("test_node", &[ResourceSlot::image_view("swapchain")], &[]);

        for _ in 0..5 {
            data.update();
            rg.get_node_mut(handle).unwrap().on_update(move || {
                println!("{}", data.value);
                Ok(())
            });
            rg.update().unwrap();
        }

        for _ in 0..5 {
            data.update();
            rg.get_node_mut("test_node").unwrap().on_update(move || {
                println!("{}", data.value);
                Ok(())
            });
            rg.update().unwrap();
        }

        assert_eq!(data.value, 10);
    }
}
