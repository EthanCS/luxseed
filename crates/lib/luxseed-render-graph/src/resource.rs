use std::borrow::Cow;

use luxseed_render_backend::{
    enums::{Format, RenderTargetLoadAction},
    flag::{BufferUsageFlags, ImageUsageFlags},
};
use luxseed_utility::pool::Handle;
use smallvec::SmallVec;

type BufferHandle = Handle<luxseed_render_backend::define::Buffer>;
type ImageViewHandle = Handle<luxseed_render_backend::define::ImageView>;
type SamplerHandle = Handle<luxseed_render_backend::define::Sampler>;

const DEFAULT_RESOURCE_SLOTS_COUNT: usize = 8;

#[derive(Debug, Clone)]
pub enum Resource {
    Buffer(Buffer),
    ImageView(ImageView),
    Sampler(Sampler),
}

impl Resource {
    #[inline]
    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::Buffer(_) => ResourceType::Buffer,
            Self::ImageView(_) => ResourceType::ImageView,
            Self::Sampler(_) => ResourceType::Sampler,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ResourceType {
    Buffer,
    ImageView,
    Sampler,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub handle: BufferHandle,
}

impl From<Buffer> for Resource {
    fn from(buffer: Buffer) -> Self {
        Self::Buffer(buffer)
    }
}

#[derive(Debug, Clone)]
pub struct ImageView {
    pub handle: ImageViewHandle,
}

impl From<ImageView> for Resource {
    fn from(image_view: ImageView) -> Self {
        Self::ImageView(image_view)
    }
}

#[derive(Debug, Clone)]
pub struct Sampler {
    pub handle: SamplerHandle,
}

impl From<Sampler> for Resource {
    fn from(sampler: Sampler) -> Self {
        Self::Sampler(sampler)
    }
}

#[derive(Debug, Clone)]
pub struct ResourceSlot {
    pub name: Cow<'static, str>,
    pub resource_type: ResourceType,
}

impl ResourceSlot {
    pub fn new(name: impl Into<Cow<'static, str>>, resource_type: ResourceType) -> Self {
        Self { name: name.into(), resource_type }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResourceSlotIdentifier {
    Name(Cow<'static, str>),
    Index(usize),
}

impl From<String> for ResourceSlotIdentifier {
    fn from(name: String) -> Self {
        Self::Name(name.into())
    }
}

impl From<&'static str> for ResourceSlotIdentifier {
    fn from(value: &'static str) -> Self {
        ResourceSlotIdentifier::Name(value.into())
    }
}

impl From<Cow<'static, str>> for ResourceSlotIdentifier {
    fn from(value: Cow<'static, str>) -> Self {
        ResourceSlotIdentifier::Name(value)
    }
}

impl From<usize> for ResourceSlotIdentifier {
    fn from(value: usize) -> Self {
        ResourceSlotIdentifier::Index(value)
    }
}

#[derive(Debug, Default)]
pub struct ResourceSlotCollection {
    slots: SmallVec<[ResourceSlot; DEFAULT_RESOURCE_SLOTS_COUNT]>,
}

impl<T: IntoIterator<Item = ResourceSlot>> From<T> for ResourceSlotCollection {
    fn from(slots: T) -> Self {
        Self { slots: slots.into_iter().collect() }
    }
}

impl ResourceSlotCollection {
    /// Returns `true` if the collection contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Returns the number of elements in the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn get_resource_slot_index(
        &self,
        identifier: impl Into<ResourceSlotIdentifier>,
    ) -> Option<usize> {
        let identifier = identifier.into();
        match identifier.into() {
            ResourceSlotIdentifier::Name(name) => self.slots.iter().position(|s| s.name == *name),
            ResourceSlotIdentifier::Index(index) => Some(index),
        }
    }

    pub fn get_resource_slot(
        &self,
        identifier: impl Into<ResourceSlotIdentifier>,
    ) -> Option<&ResourceSlot> {
        let index = self.get_resource_slot_index(identifier)?;
        self.slots.get(index)
    }

    pub fn get_resource_slot_mut(
        &mut self,
        identifier: impl Into<ResourceSlotIdentifier>,
    ) -> Option<&mut ResourceSlot> {
        let index = self.get_resource_slot_index(identifier)?;
        self.slots.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResourceSlot> {
        self.slots.iter()
    }
}
