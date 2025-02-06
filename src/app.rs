
use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::{
    entities::{TaskData, TaskEntity},
    repository::AppTreeRepository,
};

const NEW_ELEMENT_TEXT: &str = "New task.";

pub enum AppState {
    NORMAL,
    INSERT(u64),
}

pub struct App<'a> {
    pub repository: AppTreeRepository,

    pub opened_task: Option<u64>,
    pub selection_index: usize,

    pub elements_list: ListState,
    pub stack_list: ListState,
    pub text_area: TextArea<'a>,

    pub state: AppState,
}

impl App<'_> {
    pub fn new(repository: AppTreeRepository) -> Self {
        let mut list = ListState::default();
        list.select(Some(0));

        Self {
            repository,
            stack_list: ListState::default(),
            elements_list: list,
            opened_task: None,
            selection_index: 0,
            state: AppState::NORMAL,
            text_area: TextArea::default(),
        }
    }

    pub fn get_selected_task(&self) -> Option<&TaskEntity> {
        self.find_tasks_to_display().get(self.selection_index).map(|task| *task)
    }

    pub fn find_tasks_to_display(&self) -> Vec<&TaskEntity> {
        if let Some(task_id) = self.opened_task {
            self.repository.find_sub_tasks(task_id)
        } else {
            self.repository.find_root_tasks()
        }
    }

    pub fn find_parents_titles(&self) -> Vec<&str> {
        match self.opened_task {
            None => vec![],
            Some(task_id) => self
                .repository
                .find_parents_stack(task_id)
                .iter()
                .map(|task| task.title.as_str())
                .collect(),
        }
    }

    pub fn delete_current_task(&mut self) {
        let id_to_delete = self.get_selected_task().unwrap().id;
        self.repository.remove_task(id_to_delete);

        if self.selection_index > 0 {
            self.selection_index -= 1;
        }
    }

    pub fn add_new_task(&mut self) {
        let new_task = TaskData {
            title: String::from(NEW_ELEMENT_TEXT),
            children: vec![],
            done: false,
        };

        match self.opened_task {
            Some(parent_task_id) => {
                self.repository.insert_sub_task(parent_task_id, new_task);
            }
            None => {
                self.repository.insert_task(new_task);
            }
        }

        self.selection_index = self.find_tasks_to_display().len().saturating_sub(1);
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
        let task_to_nest = self.get_selected_task();
        match task_to_nest {
            None => return,
            Some(new_parent_task_id) => {
                self.opened_task = Some(new_parent_task_id.id);
                self.scroll_to_top();
            }
        }
    }

    pub fn get_back_to_parent(&mut self) {
        if self.opened_task.is_none() {
            return;
        }

        let current_parent_task_id = self.opened_task.unwrap();
        let current_parent_task_entity = self.repository.get_task(current_parent_task_id).unwrap();
        let next_parent_task_id = current_parent_task_entity.parent_id;

        self.opened_task = next_parent_task_id;
        self.scroll_to_top();
    }

    pub fn init_insert_mode_to_edit_a_task_title(&mut self) {
        let selected_task = self.get_selected_task().unwrap();

        let task_id = selected_task.id;
        let title_to_edit = selected_task.title.clone();

        self.text_area = TextArea::from([title_to_edit]);
        self.text_area.move_cursor(tui_textarea::CursorMove::End);
        self.state = AppState::INSERT(task_id);
    }

    pub fn cancel_insert_mode(&mut self) {
        self.state = AppState::NORMAL;
    }

    pub fn close_insert_mode_updating_task_title(&mut self) {
        if let AppState::INSERT(task_id) = self.state {
            let content = self.text_area.lines().join("\n");
            if !content.is_empty() {
                self.repository.update_task_title(task_id, content);
            }

            self.state = AppState::NORMAL;
        }
    }
}
