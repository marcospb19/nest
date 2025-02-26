use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::{
    entities::{ParentTask, Task, TaskData},
    history::{AppHistory, AppSnapshot},
    storage::AppStorage,
};

pub enum AppState {
    Normal,
    EditTask { task_id: u64 },
    InsertTask { parent: ParentTask },
}

pub struct App<'a> {
    pub storage: AppStorage,

    pub history: AppHistory,

    pub state: AppState,
    pub text_area: TextArea<'a>,
}

impl App<'_> {
    pub fn new(storage: AppStorage) -> Self {
        let mut elements_list = ListState::default();
        elements_list.select(Some(0));

        Self {
            storage,
            history: AppHistory::default(),
            state: AppState::Normal,
            text_area: TextArea::default(),
        }
    }

    pub fn get_or_init_selected_position(&mut self) -> usize {
        match self.storage.get_selected_position() {
            Some(position) => position,
            None => {
                self.storage.set_selected_position(0);
                0
            },
        }

    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        let selected_index = self.storage.get_selected_position()?;
        self.storage.find_opened_sub_tasks().get(selected_index).copied()
    }

    pub fn find_opened_sub_tasks(&self) -> Vec<&Task> {
        self.storage.find_opened_sub_tasks()
    }

    pub fn find_parents_titles(&self) -> Vec<&str> {
        self
            .storage
            .find_parents_stack()
            .iter()
            .map(|task| task.title.as_str())
            .collect()
    }

    pub fn find_parents_stack(&self) -> Vec<&Task> {
        self.storage.find_parents_stack()
    }

    pub fn delete_selected_task(&mut self) -> Option<u64> {
        self.save_snapshot();

        let current_position = self.storage.get_selected_position()?;

        let id_to_delete = self.get_selected_task()?.id;
        let removed_task = self.storage.remove_task(id_to_delete)?;

        self.move_selection_to(current_position.into());
        Some(removed_task.id)
    }

    pub fn move_selection_to(&mut self, index: Option<usize>) {
        let max_index = self.find_opened_sub_tasks().len().saturating_sub(1);
        let new_index = index.unwrap_or(0).max(0).min(max_index);
        self.storage.set_selected_position(new_index);
    }

    pub fn move_selection_to_top(&mut self) {
        self.move_selection_to(Some(0));
    }

    pub fn move_selection_to_bottom(&mut self) {
        let last_position = self.find_opened_sub_tasks().len().checked_sub(1);
        self.move_selection_to(last_position);
    }

    pub fn move_selection_up(&mut self) {
        let selected_position = self.storage.get_selected_position().unwrap_or(0).saturating_sub(1);
        self.move_selection_to(selected_position.into());
    }

    pub fn move_selection_down(&mut self) {
        let max_position = self.find_opened_sub_tasks().len().saturating_sub(1);
        let new_position = self.storage.get_selected_position().unwrap_or(0).saturating_add(1);
        self.move_selection_to(new_position.min(max_position).into());
    }

    pub fn swap_up(&mut self) -> Option<()> {
        let tasks = self.find_opened_sub_tasks();

        let from_index = self.storage.get_selected_position()?;
        let to_index = from_index.saturating_sub(1);

        let from_id = tasks.get(from_index)?.id;
        let to_id = tasks.get(to_index)?.id;

        if from_id != to_id {
            self.save_snapshot();
            self.storage.swap_current_sub_tasks(from_id, to_id);
            self.move_selection_up();
        }

        Some(())
    }

    pub fn swap_down(&mut self) -> Option<()> {
        let tasks = self.find_opened_sub_tasks();

        let max_index = tasks.len().saturating_sub(1);

        let from_index = self.storage.get_selected_position()?;
        let to_index = from_index.saturating_add(1).min(max_index);

        let from_id = tasks.get(from_index)?.id;
        let to_id = tasks.get(to_index)?.id;

        if from_id != to_id {
            self.save_snapshot();
            self.storage.swap_current_sub_tasks(from_id, to_id);
            self.move_selection_down();
        }
        Some(())
    }

    pub fn open_selected_task(&mut self) {
        if let Some(task) = self.get_selected_task() {
            let new_parent = ParentTask::Id(task.id);
            self.storage.view.set_opened_task(new_parent);
        }
    }

    pub fn update_done_state(&mut self) -> Option<()> {
        self.save_snapshot();
        let selected_task = self.get_selected_task()?;
        let new_done_state = !selected_task.done;
        self.storage.update_task_state(selected_task.id, new_done_state);
        Some(())
    }

    pub fn get_back_to_parent(&mut self) -> Option<()> {
        let ParentTask::Id(opened_task_id) = self.storage.get_opened_task() else {
            return None;
        };

        let current_opened_task = self.storage.get_task(opened_task_id)?;
        let new_parent_task = current_opened_task.parent;

        self.storage.set_opened_task(new_parent_task);
        Some(())
    }

    pub fn init_insert_mode_to_insert_new_task(&mut self) -> Option<()> {
        let parent = self.storage.get_opened_task();
        self.state = AppState::InsertTask { parent };
        self.text_area = TextArea::default();
        Some(())
    }

    pub fn init_insert_mode_to_edit_task_title(&mut self) -> Option<()> {
        let selected_task = self.get_selected_task()?;

        let task_id = selected_task.id;
        let title_to_edit = selected_task.title.clone();

        self.text_area = TextArea::from([title_to_edit]);
        self.text_area.move_cursor(tui_textarea::CursorMove::End);
        self.state = AppState::EditTask { task_id };
        Some(())
    }

    pub fn cancel_insert_mode(&mut self) {
        self.state = AppState::Normal;
    }

    pub fn close_insert_mode_updating_task_title(&mut self) {
        if let AppState::EditTask { task_id } = self.state {
            self.state = AppState::Normal;
            let content = self.text_area.lines().join("\n");

            if content.is_empty() {
                return;
            }

            self.save_snapshot();
            self.storage.update_task_title(task_id, content);
        }
    }

    pub fn close_insert_mode_inserting_new_task(&mut self) {
        if let AppState::InsertTask { parent } = self.state {
            self.state = AppState::Normal;
            let content = self.text_area.lines().join("\n");

            if content.is_empty() {
                return;
            }

            self.save_snapshot();

            let task_data = TaskData {
                title: content,
                children: vec![],
                done: false,
            };

            self.storage.insert_task(parent, task_data);

            self.move_selection_to_bottom();
        }
    }

    pub fn save_snapshot(&mut self) {
        let snapshot = self.create_snapshot();
        self.history.save_snapshot(snapshot);
    }

    pub fn undo(&mut self) -> Option<()> {
        let current_snapshot = self.create_snapshot();
        let snapshot_to_restore = self.history.undo(current_snapshot)?;
        self.restore_snapshot(snapshot_to_restore);
        Some(())
    }

    pub fn redo(&mut self) -> Option<()> {
        let current_snapshot = self.create_snapshot();
        let snapshot_to_restore = self.history.redo(current_snapshot)?;
        self.restore_snapshot(snapshot_to_restore);
        Some(())
    }

    pub fn create_snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            tasks: self.storage.tasks.clone(),
            opened_task: self.storage.get_opened_task(),
            selected_index: self.storage.get_selected_position(),
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: AppSnapshot) {
        self.storage.tasks = snapshot.tasks;
        self.storage.view.set_opened_task(snapshot.opened_task);
        self.move_selection_to(snapshot.selected_index);
    }
}
