use indexmap::IndexMap;
use crate::{app::App, entities::Task};

#[derive(Clone)]
pub struct AppSnapshot {
    pub tasks: IndexMap<u64, Task>,
    pub opened_task: Option<u64>,
    pub selected_index: Option<usize>,
}

#[derive(Default)]
pub struct AppHistory {
    pub history: Vec<AppSnapshot>,
    pub cursor: Option<usize>,
}

impl AppHistory {
    pub fn save_snapshot(&mut self, snapshot: AppSnapshot) {
        if let Some(cursor) = self.cursor {
            self.history.truncate(cursor + 1);
        }

        self.history.push(snapshot);
        self.cursor = Some(self.history.len() - 1);
    }

    pub fn undo(&mut self) -> Option<AppSnapshot> {
        let new_cursor = self.cursor?.checked_sub(1)?;
        let snapshot_to_apply = self.history.get(new_cursor)?.clone();

        self.cursor = Some(new_cursor);
        Some(snapshot_to_apply)
    }

    pub fn redo(&mut self) -> Option<AppSnapshot> {
        if self.cursor? == self.history.len() - 1 {
            return None;
        }

        let new_cursor = self.cursor? + 1;
        let snapshot_to_apply = self.history.get(new_cursor)?.clone();

        self.cursor = Some(new_cursor);
        Some(snapshot_to_apply)
    }
}