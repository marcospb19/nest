use indexmap::IndexMap;
use crate::{app::App, entities::Task};

#[derive(Clone)]
pub struct AppSnapshot {
    pub tasks: IndexMap<u64, Task>,
    pub opened_task: Option<u64>,
    pub selected_index: Option<usize>,
}

