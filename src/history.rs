use indexmap::IndexMap;

use crate::entities::{ParentTask, Task};

#[derive(Clone)]
pub struct AppSnapshot {
    pub tasks: IndexMap<u64, Task>,
    pub opened_task: ParentTask,
    pub selected_index: Option<usize>,
}

#[derive(Default)]
pub struct AppHistory {
    pub undo_stack: Vec<AppSnapshot>,
    pub redo_stack: Vec<AppSnapshot>,
}

impl AppHistory {
    pub fn save_snapshot(&mut self, snapshot: AppSnapshot) {
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_snapshot: AppSnapshot) -> Option<AppSnapshot> {
        let snapshot_to_restore = self.undo_stack.pop()?;
        self.redo_stack.push(current_snapshot);
        Some(snapshot_to_restore)
    }

    pub fn redo(&mut self, current_snapshot: AppSnapshot) -> Option<AppSnapshot> {
        let snapshot_to_restore = self.redo_stack.pop()?;
        self.undo_stack.push(current_snapshot);
        Some(snapshot_to_restore)
    }
}
