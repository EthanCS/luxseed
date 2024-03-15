use crate::archetype::TypeMeta;

pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

pub trait ComponentBundle {
    fn type_infos(&self) -> Vec<TypeMeta>;
}
