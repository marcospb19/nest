use std::collections::HashMap;

use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::{
    entities::{Task, TaskData},
    storage::AppTreeStorage,
};

pub enum AppState {
    Normal,
    EditTask { task_id: u64 },
    InsertTask { parent_id: Option<u64> },
}

pub struct App<'a> {
    pub storage: AppTreeStorage,

    pub opened_task: Option<u64>,
    pub selections_in_tasks: HashMap<Option<u64>, usize>,
    
    pub state: AppState,
    pub text_area: TextArea<'a>,
}

impl App<'_> {
    pub fn new(storage: AppTreeStorage) -> Self {
        let mut elements_list = ListState::default();
        elements_list.select(Some(0));

        Self {
            storage,
            selections_in_tasks: HashMap::new(),
            opened_task: None,
            state: AppState::Normal,
            text_area: TextArea::default(),
        }
    }

    pub fn get_position_selected_task(&self) -> Option<usize> {
        self.selections_in_tasks.get(&self.opened_task).cloned()
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        let selected_index = self.get_position_selected_task()?;
        self.find_tasks_to_display().get(selected_index).copied()
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

    pub fn delete_selected_task(&mut self) -> Option<u64> {
        let current_position = self.get_position_selected_task()?;

        let id_to_delete = self.get_selected_task()?.id;
        let removed_task = self.storage.remove_task(id_to_delete)?;

        self.move_selection_to(current_position.into());
        Some(removed_task.id)
    }

    pub fn move_selection_to(&mut self, index: Option<usize>) {
        let max_index = self.find_tasks_to_display().len().saturating_sub(1);
        let new_index = index.unwrap_or(0).max(0).min(max_index);
        self.selections_in_tasks.insert(self.opened_task, new_index);
    }

    pub fn move_selection_to_top(&mut self) {
        self.move_selection_to(Some(0))
    }

    pub fn move_selection_to_bottom(&mut self) {
        let last_position = self.find_tasks_to_display().len().checked_sub(1);
        self.move_selection_to(last_position);
    }

    pub fn move_selection_up(&mut self) {
        let selected_position = self.get_position_selected_task().unwrap_or(0).saturating_sub(1);
        self.move_selection_to(selected_position.into());
    }

    pub fn move_selection_down(&mut self) {
        let max_position = self.find_tasks_to_display().len().saturating_sub(1);
        let new_position = self.get_position_selected_task().unwrap_or(0).saturating_add(1);
        self.move_selection_to(new_position.min(max_position).into());
    }

    pub fn swap_up(&mut self) -> Option<()> {
        let parent_id = self.opened_task;

        let tasks = self.find_tasks_to_display();

        let from_index = self.get_position_selected_task()?;
        let to_index = from_index.saturating_sub(1);

        let from_id = tasks.get(from_index)?.id;
        let to_id = tasks.get(to_index)?.id;

        if from_id != to_id {
            self.storage.swap_sub_tasks(parent_id, from_id, to_id);
            self.move_selection_up();
        }

        Some(())
    }

    pub fn swap_down(&mut self) -> Option<()> {
        let parent_id = self.opened_task;

        let tasks = self.find_tasks_to_display();

        let max_index = tasks.len().saturating_sub(1);

        let from_index = self.get_position_selected_task()?;
        let to_index = from_index.saturating_add(1).min(max_index);

        let from_id = tasks.get(from_index)?.id;
        let to_id = tasks.get(to_index)?.id;

        if from_id != to_id {
            self.storage.swap_sub_tasks(parent_id, from_id, to_id);
            self.move_selection_down();
        }
        Some(())
    }

    pub fn open_selected_task(&mut self) {
        if let Some(new_parent_task_id) = self.get_selected_task() {
            self.opened_task = Some(new_parent_task_id.id);
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
                Some(parent_id) => self.storage.insert_sub_task(parent_id, task_data),
                None => self.storage.insert_task(task_data),
            }

            self.move_selection_to_bottom();
        }
    }
}
