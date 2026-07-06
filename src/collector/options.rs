use crate::app::options::DetailLevel;

#[derive(Debug, Clone, Copy)]
pub struct CollectOptions {
    pub detail: DetailLevel,
    pub powershell: bool,
}
