use anyhow::{Context, Result};
use std::{borrow::Cow, collections::HashMap};

use crate::{
    edge::Edge,
    node::{Node, NodeHandle, NodeIdentifier},
    resource::{ResourceSlot, ResourceSlotIdentifier},
};

const MAX_RESOURCES_COUNT: usize = 1024;
const MAX_NODES_COUNT: usize = 1024;

#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<NodeHandle, Node>,
    node_names: HashMap<Cow<'static, str>, NodeHandle>,
}

impl RenderGraph {
    pub fn update(&mut self) -> Result<()> {
        for node in self.nodes.values_mut() {
            if let Some(on_update) = node.on_update.take() {
                on_update()?;
            }
        }
        Ok(())
    }

    pub fn get_node_handle(&self, identifier: impl Into<NodeIdentifier>) -> Result<NodeHandle> {
        match identifier.into() {
            NodeIdentifier::Name(ref name) => self
                .node_names
                .get(name)
                .copied()
                .context(anyhow::format_err!("Node with name {:?} not found", name)),
            NodeIdentifier::Handle(handle) => Ok(handle),
        }
    }

    pub fn get_node(&self, identifier: impl Into<NodeIdentifier>) -> Result<&Node> {
        let node_handle = self.get_node_handle(identifier)?;
        self.nodes
            .get(&node_handle)
            .context(anyhow::format_err!("Node with handle {:?} not found", node_handle))
    }

    pub fn get_node_mut(&mut self, identifier: impl Into<NodeIdentifier>) -> Result<&mut Node> {
        let node_handle = self.get_node_handle(identifier)?;
        self.nodes
            .get_mut(&node_handle)
            .context(anyhow::format_err!("Node with handle {:?} not found", node_handle))
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

    pub fn remove_node(&mut self, name: impl Into<Cow<'static, str>>) -> Result<()> {
        todo!()
    }

    /// Checks if the graph has this edge.
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

    pub fn try_add_node_edge(
        &mut self,
        output: impl Into<NodeIdentifier>,
        input: impl Into<NodeIdentifier>,
    ) -> Result<()> {
        let output = self.get_node_handle(output)?;
        let input = self.get_node_handle(input)?;

        let new_edge = Edge::NodeEdge { output_node: output, input_node: input };
        if !self.has_edge(&new_edge) {
            {
                let output_node = self.get_node_mut(output)?;
                output_node.add_output_edge(new_edge)?;
            }
            let input_node = self.get_node_mut(input)?;
            input_node.add_input_edge(new_edge)?;
        }

        Ok(())
    }

    pub fn remove_node_edge(
        &mut self,
        output: impl Into<NodeIdentifier>,
        input: impl Into<NodeIdentifier>,
    ) -> Result<()> {
        todo!()
    }

    pub fn try_add_resource_edge(
        &mut self,
        output_node: impl Into<NodeIdentifier>,
        output_slot: impl Into<ResourceSlotIdentifier>,
        input_node: impl Into<NodeIdentifier>,
        input_slot: impl Into<ResourceSlotIdentifier>,
    ) -> Result<()> {
        todo!()
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
