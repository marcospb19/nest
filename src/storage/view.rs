use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entities::ParentTask;


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ViewStorage {
    pub opened_task: ParentTask,
    pub selections_in_tasks: HashMap<ParentTask, usize>,
}

impl ViewStorage {
    pub fn get_opened_task(&self) -> ParentTask {
        self.opened_task
    }

    pub fn set_opened_task(&mut self, opened_task: ParentTask) {
        self.opened_task = opened_task;
    }

    pub fn get_position_selected_task(&self) -> Option<usize> {
        self.selections_in_tasks.get(&self.opened_task).cloned()
    }

    pub fn set_position_selected_task(&mut self, index: usize) {
        self.selections_in_tasks.insert(self.opened_task, index);
    }
}