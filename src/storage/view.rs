use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entities::ParentTask;


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ViewStorage {
    pub opened_task: ParentTask,
    pub positions_in_opened_task: HashMap<ParentTask, usize>,
}

impl ViewStorage {
    pub fn get_opened_task(&self) -> ParentTask {
        self.opened_task
    }

    pub fn set_opened_task(&mut self, opened_task: ParentTask) {
        self.opened_task = opened_task;
    }

    pub fn get_selected_position(&self) -> Option<usize> {
        self.positions_in_opened_task.get(&self.opened_task).cloned()
    }

    pub fn set_selected_position(&mut self, index: usize) {
        self.positions_in_opened_task.insert(self.opened_task, index);
    }
}