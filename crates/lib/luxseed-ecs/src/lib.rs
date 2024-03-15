use thiserror::Error;

pub mod archetype;
pub mod bundle;
pub mod component;
pub mod entity;
pub mod entity_ref;
pub mod storage;
pub mod unsafe_world_cell;
pub mod world;

#[derive(Error, Debug)]
pub enum EcsError {
    #[error("entity not found")]
    EntityNotFound,
    #[error("missing component")]
    MissingComponent,
}
