// use std::{alloc::Layout, any::TypeId, borrow::Cow, collections::HashMap};

// pub trait Component: 'static + Send + Sync {}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
// pub struct ComponentId(usize);

// #[derive(Debug, Clone)]
// pub struct ComponentDescriptor {
//     name: Cow<'static, str>,
//     type_id: Option<TypeId>,
//     layout: Layout,
// }

// impl ComponentDescriptor {
//     pub fn new<T: Component>() -> Self {
//         Self {
//             name: Cow::Borrowed(std::any::type_name::<T>()),
//             type_id: Some(TypeId::of::<T>()),
//             layout: Layout::new::<T>(),
//         }
//     }

//     #[inline]
//     pub fn name(&self) -> &str {
//         &self.name
//     }

//     #[inline]
//     pub fn type_id(&self) -> Option<TypeId> {
//         self.type_id
//     }
// }

// #[derive(Debug, Clone)]
// pub struct ComponentInfo {
//     id: ComponentId,
//     descriptor: ComponentDescriptor,
// }

// impl ComponentInfo {
//     #[inline]
//     pub fn id(&self) -> ComponentId {
//         self.id
//     }

//     #[inline]
//     pub fn name(&self) -> &str {
//         &self.descriptor.name
//     }

//     #[inline]
//     pub fn type_id(&self) -> Option<TypeId> {
//         self.descriptor.type_id
//     }

//     #[inline]
//     pub fn layout(&self) -> Layout {
//         self.descriptor.layout
//     }

//     pub(crate) fn new(id: ComponentId, descriptor: ComponentDescriptor) -> Self {
//         ComponentInfo { id, descriptor }
//     }
// }

// #[derive(Debug, Default)]
// pub struct Components {
//     components: Vec<ComponentInfo>,
//     type_id_to_component_id: HashMap<TypeId, ComponentId>,
// }

// impl Components {
//     #[inline]
//     pub fn init_component<T: Component>(&mut self) -> ComponentId {
//         let type_id = TypeId::of::<T>();
//         *self.type_id_to_component_id.entry(type_id).or_insert_with(|| {
//             let id = ComponentId(self.components.len());
//             self.components.push(ComponentInfo::new(id, ComponentDescriptor::new::<T>()));
//             id
//         })
//     }

//     #[inline]
//     pub fn len(&self) -> usize {
//         self.components.len()
//     }

//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.components.is_empty()
//     }

//     #[inline]
//     pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
//         self.components.get(id.0)
//     }

//     #[inline]
//     pub fn get_id(&self, type_id: TypeId) -> Option<ComponentId> {
//         self.type_id_to_component_id.get(&type_id).copied()
//     }

//     #[inline]
//     pub fn component_id<T: Component>(&self) -> Option<ComponentId> {
//         self.get_id(TypeId::of::<T>())
//     }
// }

// pub trait ComponentVec {
//     fn as_any(&self) -> &dyn std::any::Any;
//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
//     fn push_none(&mut self);
// }

// impl<T: 'static> ComponentVec for Vec<Option<T>> {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }
//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
//     fn push_none(&mut self) {
//         self.push(None);
//     }
// }

use std::any::Any;

use crate::storage::UnsafeStorage;

pub trait Component: Any + Sized {
    type Storage: UnsafeStorage<Self>;
}
