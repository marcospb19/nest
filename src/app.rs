use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::{
    entities::{Task, TaskData},
    storage::AppTreeStorage,
};

const NEW_ELEMENT_TEXT: &str = "New task.";

pub enum AppState {
    Normal,
    EditTask { task_id: u64 },
    InsertTask { parent_id: Option<u64> },
}

pub struct App<'a> {
    pub storage: AppTreeStorage,

    pub opened_task: Option<u64>,
    pub selection_index: usize,

    pub elements_list: ListState,
    pub stack_list: ListState,
    pub text_area: TextArea<'a>,

    pub state: AppState,
}

impl App<'_> {
    pub fn new(storage: AppTreeStorage) -> Self {
        let mut elements_list = ListState::default();
        elements_list.select(Some(0));

        Self {
            storage,
            elements_list,
            stack_list: ListState::default(),
            opened_task: None,
            selection_index: 0,
            state: AppState::Normal,
            text_area: TextArea::default(),
        }
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        self.find_tasks_to_display().get(self.selection_index).copied()
    }

    pub fn find_tasks_to_display(&self) -> Vec<&Task> {
        if let Some(task_id) = self.opened_task {
            self.storage.find_sub_tasks(task_id)
        } else {
            self.storage.find_root_tasks()
        }
    }

    pub fn find_parents_titles(&self) -> Vec<&str> {
        match self.opened_task {
            None => vec![],
            Some(task_id) => self
                .storage
                .find_parents_stack(task_id)
                .iter()
                .map(|task| task.title.as_str())
                .collect(),
        }
    }

    pub fn delete_current_task(&mut self) -> Option<u64> {
        let id_to_delete = self.get_selected_task()?.id;

        if self.selection_index > 0 {
            self.selection_index -= 1;
        }

        self.storage.remove_task(id_to_delete).map(|task| task.id)
    }

    pub fn add_new_task(&mut self, title: String) {
        let new_task = TaskData {
            title,
            children: vec![],
            done: false,
        };

        match self.opened_task {
            Some(parent_task_id) => {
                self.storage.insert_sub_task(parent_task_id, new_task);
            }
            None => {
                self.storage.insert_task(new_task);
            }
        }

        self.move_display_to(self.find_tasks_to_display().len().checked_sub(1));
    }

    pub fn move_display_to(&mut self, index: Option<usize>) {
        let max_index = self.find_tasks_to_display().len().saturating_sub(1);
        let index = index.filter(|n| *n <= max_index);
        self.selection_index = index.unwrap_or(0);
        self.elements_list.select(index);
    }

    pub fn scroll_to_top(&mut self) {
        self.move_display_to(Some(0))
    }

    pub fn scroll_to_bottom(&mut self) {
        self.move_display_to(self.find_tasks_to_display().len().checked_sub(1));
    }

    pub fn move_selection_up(&mut self) {
        self.move_display_to(self.selection_index.checked_sub(1));
    }

    pub fn move_selection_down(&mut self) {
        let max_index = self.find_tasks_to_display().len().saturating_sub(1);
        let new_index = self.selection_index + 1;
        self.move_display_to(Some(new_index.min(max_index)));
    }

    pub fn nest_task(&mut self) {
        if let Some(new_parent_task_id) = self.get_selected_task() {
            self.opened_task = Some(new_parent_task_id.id);
            self.scroll_to_top();
        }
    }

    pub fn update_done_state(&mut self) {
        let selected_task = self.get_selected_task().unwrap();
        let new_done_state = !selected_task.done;
        self.storage.update_task_state(selected_task.id, new_done_state);
    }

    pub fn get_back_to_parent(&mut self) -> Option<()> {
        let current_parent_task_id = self.opened_task?;
        let current_parent_task = self.storage.get_task(current_parent_task_id)?;
        let next_parent_task_id = current_parent_task.parent_id;

        self.opened_task = next_parent_task_id;
        self.scroll_to_top();
        Some(())
    }

    pub fn init_insert_mode_to_insert_new_task(&mut self) -> Option<()> {
        let parent_id = self.opened_task;
        self.state = AppState::InsertTask { parent_id };
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

            self.storage.update_task_title(task_id, content);
        }
    }

    pub fn close_insert_mode_inserting_new_task(&mut self) {
        if let AppState::InsertTask { parent_id } = self.state {
            self.state = AppState::Normal;
            let content = self.text_area.lines().join("\n");

            if content.is_empty() {
                return;
            }

            let task_data = TaskData {
                title: content,
                children: vec![],
                done: false,
            };

            match parent_id {
                Some(parent_id) => self.storage.insert_sub_task(parent_id,task_data),
                None => self.storage.insert_task(task_data),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app() {
        let storage = AppTreeStorage::default();
        let app = App::new(storage);

        assert_eq!(app.selection_index, 0);
        assert!(matches!(app.state, AppState::Normal));
        assert!(app.opened_task.is_none());
    }

    #[test]
    fn test_add_new_task() {
        let storage = AppTreeStorage::default();
        let mut app = App::new(storage);

        app.add_new_task();
        let tasks = app.find_tasks_to_display();
        assert_eq!(app.opened_task, None);
        assert_eq!(tasks.len(), 1);

        app.add_new_task();
        app.add_new_task();

        let tasks = app.find_tasks_to_display();
        assert_eq!(app.opened_task, None);
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn test_delete_current_task() {
        let storage = AppTreeStorage::default();
        let mut app = App::new(storage);

        app.add_new_task();
        let deleted_task_id = app.delete_current_task();

        assert!(deleted_task_id.is_some());
        assert!(app.find_tasks_to_display().is_empty());
    }

    #[test]
    fn test_nest_task() {
        let storage = AppTreeStorage::default();
        let mut app = App::new(storage);

        app.add_new_task();
        app.nest_task();

        assert!(app.opened_task.is_some());
    }

    #[test]
    fn test_get_back_to_parent() {
        let storage = AppTreeStorage::default();
        let mut app = App::new(storage);

        app.add_new_task();
        app.nest_task();

        let parent_id = app.opened_task;
        assert!(app.get_back_to_parent().is_some());
        assert_ne!(app.opened_task, parent_id);
    }

    #[test]
    fn test_init_insert_mode() {
        let storage = AppTreeStorage::default();
        let mut app = App::new(storage);

        app.add_new_task();
        app.init_insert_mode_to_edit_a_task_title();

        assert!(matches!(app.state, AppState::EditTask { .. }));
    }

    #[test]
    fn test_cancel_insert_mode() {
        let storage = AppTreeStorage::default();
        let mut app = App::new(storage);

        app.add_new_task();
        app.init_insert_mode_to_edit_a_task_title();
        app.cancel_insert_mode();

        assert!(matches!(app.state, AppState::Normal));
    }
}
