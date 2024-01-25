use luxseed_utility::define_atomic_id;
use smallvec::SmallVec;
use std::{borrow::Cow, fmt::Debug, fmt::Formatter};

use crate::{context::RenderGraphContext, edge::Edge, resource::ResourceSlotCollection};

const DEFAULT_RESOURCES_COUNT: usize = 4;
const DEFAULT_EDGES_COUNT: usize = 4;

define_atomic_id!(NodeHandle);

type OnUpdateFn = dyn FnOnce() -> anyhow::Result<()>;
type OnRenderFn = dyn FnOnce(&mut RenderGraphContext) -> anyhow::Result<()>;

pub struct Node {
    pub handle: NodeHandle,
    pub name: Option<Cow<'static, str>>,
    pub on_update: Option<Box<OnUpdateFn>>,
    pub on_render: Option<Box<OnRenderFn>>,
    input_edges: SmallVec<[Edge; DEFAULT_EDGES_COUNT]>,
    output_edges: SmallVec<[Edge; DEFAULT_EDGES_COUNT]>,
    input_resource_slots: ResourceSlotCollection,
    output_resource_slots: ResourceSlotCollection,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} ({:?})", self.handle, self.name)
    }
}

impl Node {
    pub fn new(handle: NodeHandle) -> Self {
        Self {
            handle,
            name: None,
            input_edges: SmallVec::new(),
            input_resource_slots: Default::default(),
            output_edges: SmallVec::new(),
            output_resource_slots: Default::default(),
            on_update: None,
            on_render: None,
        }
    }

    #[inline]
    pub fn input_edges(&self) -> &[Edge] {
        &self.input_edges
    }

    #[inline]
    pub fn output_edges(&self) -> &[Edge] {
        &self.output_edges
    }

    #[inline]
    pub fn has_input_edge(&self, edge: &Edge) -> bool {
        self.input_edges.contains(edge)
    }

    #[inline]
    pub fn has_output_edge(&self, edge: &Edge) -> bool {
        self.output_edges.contains(edge)
    }

    pub fn add_input_edge(&mut self, edge: Edge) -> anyhow::Result<()> {
        if self.has_input_edge(&edge) {
            return Err(anyhow::anyhow!("Input edge already exists"));
        }
        self.input_edges.push(edge);
        Ok(())
    }

    pub fn add_output_edge(&mut self, edge: Edge) -> anyhow::Result<()> {
        if self.has_output_edge(&edge) {
            return Err(anyhow::anyhow!("Output edge already exists"));
        }
        self.output_edges.push(edge);
        Ok(())
    }

    pub fn remove_input_edge(&mut self, edge: Edge) -> anyhow::Result<()> {
        if let Some(index) = self.input_edges.iter().position(|e| *e == edge) {
            self.input_edges.remove(index);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Input edge not found"))
        }
    }

    pub fn remove_output_edge(&mut self, edge: Edge) -> anyhow::Result<()> {
        if let Some(index) = self.output_edges.iter().position(|e| *e == edge) {
            self.output_edges.remove(index);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Output edge not found"))
        }
    }

    pub fn on_update(&mut self, on_update: impl (FnOnce() -> anyhow::Result<()>) + 'static) {
        let prev = self.on_update.replace(Box::new(on_update));
        assert!(prev.is_none());
    }

    pub fn on_render(
        &mut self,
        on_render: impl (FnOnce(&mut RenderGraphContext) -> anyhow::Result<()>) + 'static,
    ) {
        let prev = self.on_render.replace(Box::new(on_render));
        assert!(prev.is_none());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeIdentifier {
    Name(Cow<'static, str>),
    Handle(NodeHandle),
}

impl From<&'static str> for NodeIdentifier {
    fn from(name: &'static str) -> Self {
        Self::Name(name.into())
    }
}

impl From<String> for NodeIdentifier {
    fn from(name: String) -> Self {
        Self::Name(name.into())
    }
}

impl From<NodeHandle> for NodeIdentifier {
    fn from(handle: NodeHandle) -> Self {
        Self::Handle(handle)
    }
}

impl From<&NodeIdentifier> for NodeIdentifier {
    fn from(identifier: &NodeIdentifier) -> Self {
        identifier.clone()
    }
}
