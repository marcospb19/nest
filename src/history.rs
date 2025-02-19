
use indexmap::IndexMap;
use crate::entities::Task;

#[derive(Clone)]
pub struct AppSnapshot {
    pub tasks: IndexMap<u64, Task>,
    pub opened_task: Option<u64>,
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

    pub fn undo(&mut self) -> Option<AppSnapshot> {
        let snapshot = self.undo_stack.pop()?;
        self.redo_stack.push(snapshot.clone());
        Some(snapshot)
    }

    pub fn redo(&mut self) -> Option<AppSnapshot> {
        let snapshot = self.redo_stack.pop()?;
        self.undo_stack.push(snapshot.clone());
        Some(snapshot)
    }
}