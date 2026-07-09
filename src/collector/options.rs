use crate::app::options::{Backend, DetailLevel};

#[derive(Debug, Clone, Copy)]
pub struct CollectOptions {
    pub detail: DetailLevel,
    pub backend: Backend,
    pub debug: bool,
}
