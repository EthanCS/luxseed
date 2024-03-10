use thiserror::Error;

pub mod archetype;
pub mod bundle;
pub mod component;
pub mod entity;
pub mod storage;
pub mod world;

#[derive(Error, Debug)]
pub enum EcsError {
    #[error("entity not found")]
    EntityNotFound,
}
