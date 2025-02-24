use serde::{Deserialize, Serialize};

use crate::entities::ParentTask;


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ViewStorage {
    pub opened_task: ParentTask,
    pub positions_in_opened_task: Vec<(ParentTask, usize)>,
}

impl ViewStorage {
    pub fn get_opened_task(&self) -> ParentTask {
        self.opened_task
    }

    pub fn set_opened_task(&mut self, opened_task: ParentTask) {
        self.opened_task = opened_task;
    }

    pub fn get_selected_position(&self) -> Option<usize> {
        self.positions_in_opened_task
            .iter()
            .find(|p| p.0 == self.opened_task)
            .map(|p| p.1)
    }

    pub fn set_selected_position(&mut self, index: usize) {
        self.positions_in_opened_task.retain(|p| p.0 != self.opened_task);
        self.positions_in_opened_task.push((self.opened_task, index));
    }
}