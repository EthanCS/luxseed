use anyhow::{Context, Result};
use std::{borrow::Cow, collections::HashMap};

use crate::node::{Node, NodeHandle, NodeIdentifier};

const MAX_RESOURCES_COUNT: usize = 1024;
const MAX_NODES_COUNT: usize = 1024;

#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<NodeHandle, Node>,
    node_names: HashMap<Cow<'static, str>, NodeHandle>,
}

impl RenderGraph {
    pub fn update(&mut self) -> anyhow::Result<()> {
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

    pub fn add_node(&mut self, name: impl Into<Cow<'static, str>>) -> NodeHandle {
        let handle = NodeHandle::new();
        let name = name.into();
        let mut node = Node::new(handle);
        node.name = Some(name.clone());
        self.nodes.insert(handle, node);
        self.node_names.insert(name, handle);
        handle
    }

    pub fn add_edge(&mut self, edges: &[&'static str]) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
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
        let handle = rg.add_node("test_node");

        for _ in 0..5 {
            {
                data.update();
                rg.get_node_mut(handle).unwrap().on_update(move || {
                    println!("{}", data.value);
                    Ok(())
                });
                rg.update().unwrap();
            }
            {
                data.update();
                rg.get_node_mut("test_node").unwrap().on_update(move || {
                    println!("{}", data.value);
                    Ok(())
                });
                rg.update().unwrap();
            }
        }
    }
}
