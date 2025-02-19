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
    pub fn save_snapshot(&mut self, app: &App) {
        let snapshot = app.create_snapshot();
        
        if let Some(cursor) = self.cursor {
            self.history.truncate(cursor);
        }

        self.history.push(snapshot);
        self.cursor = Some(self.history.len() - 1);
    }

    pub fn undo(&mut self, app: &mut App) -> Option<()> {
        let new_cursor = self.cursor?.checked_sub(1)?;
        let snapshot_to_apply = self.history.get(new_cursor)?.clone();

        self.cursor = Some(new_cursor);
        app.restore_snapshot(snapshot_to_apply);

        Some(())
    }

    pub fn redo(&mut self, app: &mut App) -> Option<()> {
        if self.cursor? == self.history.len() - 1 {
            return None;
        }

        let new_cursor = self.cursor? + 1;
        let snapshot_to_apply = self.history.get(new_cursor)?.clone();

        self.cursor = Some(new_cursor);
        app.restore_snapshot(snapshot_to_apply);

        Some(())
    }
}