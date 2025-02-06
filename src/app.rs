use std::task;

use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::{entities::{TaskData, TaskEntity}, repository::{self, AppTreeRepository}};

const PLACEHOLDER_TEXT: &str = "Placeholder text.";
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
            Some(task_id) => {
                self.repository
                    .find_parents_stack(task_id)
                    .iter()
                    .map(|task| task.title.as_str())
                    .collect()
            },
        }
    }

    pub fn delete_current_task(&mut self) {
        let id_to_delete = self.get_selected_task().unwrap().id;
        self.repository.remove_task(id_to_delete);

        if self.selection_index > 0  {
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
            },
            None => {
                self.repository.insert_task(new_task);
            },
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

        self.text_area = TextArea::from([ title_to_edit ]);
        self.state = AppState::INSERT(task_id);
    }

    pub fn cancel_insert_mode(&mut self) {
        self.state = AppState::NORMAL;
    }

    pub fn close_insert_mode_updating_task_title(&mut self) {
        if let AppState::INSERT(task_id) = self.state {
            let content = self.text_area.lines().join("\n");
            self.repository.update_task_title(task_id, content);
            self.state = AppState::NORMAL;
        }
    }

    // pub fn find_nodes_in_view(&self) -> &[TaskEntity] {
    //     match self.parent_view_path.len() {
    //         0 => &self.trees,
    //         1 => &self.trees[self.parent_view_path[0]].children,
    //         _ => {
    //             &self.trees[self.parent_view_path[0]]
    //                 .get_node(&self.parent_view_path[1..])
    //                 .expect("parent_view_path must be valid")
    //                 .children
    //         }
    //     }
    // }

    // pub fn parent_node(&self) -> &ElementTree {
    //     let path = &self.parent_view_path;
    //     if path.len() == 1 {
    //         &self.trees[path[0]]
    //     } else {
    //         self.trees[path[0]]
    //             .get_node(&path[1..])
    //             .expect("parent_view_path must be valid")
    //     }
    // }

    // pub fn selection_index(&self) -> usize {
    //     self.elements_list.selected().unwrap()
    // }

    // #[expect(unused)]
    // fn selected_node(&self) -> &ElementTree {
    //     &self.nodes_in_view()[self.selection_index()]
    // }

    // pub fn update_selection(&mut self, index: usize) {
    //     self.elements_list.select(Some(index));
    // }

    // fn path_to_selection(&self) -> Vec<usize> {
    //     self.parent_view_path
    //         .iter()
    //         .copied()
    //         .chain([self.selection_index()])
    //         .collect()
    // }

    // pub fn move_selection_up(&mut self) {
    //     let next = if self.selection_index() == 0 {
    //         self.trees.len() - 1 // Wrap to end
    //     } else {
    //         self.selection_index() - 1
    //     };
    //     self.update_selection(next);
    // }

    // pub fn move_selection_down(&mut self) {
    //     let next = if self.selection_index() + 1 >= self.trees.len() {
    //         0 // Wrap to start
    //     } else {
    //         self.selection_index() + 1
    //     };
    //     self.update_selection(next);
    // }

    // pub fn add_element_below(&mut self) {
    //     self.move_selection_down();
    //     let tree_path = self.path_to_selection();
    //     self.insert_at(&tree_path, ElementTree::new_leaf(PLACEHOLDER_TEXT));
    //     self.history.push_operation(HistoryOperation::AddElement {
    //         text: PLACEHOLDER_TEXT.to_string(),
    //         tree_path,
    //     });
    // }

    // pub fn delete_element(&mut self) {
    //     if self.trees.len() == 1 {
    //         if self.trees[0] == ElementTree::new_leaf(PLACEHOLDER_TEXT) {
    //             return;
    //         }
    //         self.add_element_below();
    //         self.move_selection_up();
    //     }

    //     let tree_path = self.path_to_selection();
    //     let deleted_node = self.delete_at(&tree_path);
    //     let deletion = HistoryOperation::DeleteElement {
    //         text: deleted_node.text,
    //         tree_path,
    //         children: deleted_node.children,
    //     };

    //     self.history.push_operation(deletion);
    // }

    // fn delete_at(&mut self, tree_path: &[usize]) -> ElementTree {
    //     if tree_path.len() == 1 {
    //         self.trees.remove(tree_path[0])
    //     } else {
    //         self.trees[tree_path[0]].remove_node(&tree_path[1..])
    //     }
    // }

    // fn insert_at(&mut self, tree_path: &[usize], new_leaf: ElementTree) {
    //     if tree_path.len() == 1 {
    //         self.trees.insert(tree_path[0], new_leaf);
    //     } else {
    //         self.trees[tree_path[0]].insert_node(&tree_path[1..], new_leaf);
    //     }
    // }

    // pub fn scroll_to_top(&mut self) {
    //     self.update_selection(0);
    // }

    // pub fn scroll_to_bottom(&mut self) {
    //     self.update_selection(self.trees.len() - 1);
    // }

    // pub fn undo_change(&mut self) {
    //     let Some(operation) = self.history.undo_operation() else {
    //         return;
    //     };
    //     match operation {
    //         HistoryOperation::AddElement { tree_path, .. } => {
    //             self.delete_at(&tree_path);

    //             // if edit happened above selection, move the selection
    //             if tree_path
    //                 .get(self.parent_view_path.len())
    //                 .is_some_and(|&position| self.selection_index() >= position)
    //             {
    //                 self.move_selection_up();
    //             }
    //         }
    //         HistoryOperation::DeleteElement {
    //             text,
    //             tree_path,
    //             children,
    //         } => {
    //             self.insert_at(&tree_path, ElementTree::new_branch(text, children));

    //             // if edit happened above selection, move the selection
    //             if tree_path
    //                 .get(self.parent_view_path.len())
    //                 .is_some_and(|&position| self.selection_index() >= position)
    //             {
    //                 self.move_selection_down();
    //             }
    //         }
    //     }
    // }

    // pub fn redo_change(&mut self) {
    //     let Some(operation) = self.history.redo_operation() else {
    //         return;
    //     };
    //     match operation {
    //         HistoryOperation::AddElement { text, tree_path } => {
    //             self.insert_at(&tree_path, ElementTree::new_branch(text, vec![]));

    //             // if edit happened above selection, move the selection
    //             if tree_path
    //                 .get(self.parent_view_path.len())
    //                 .is_some_and(|&position| self.selection_index() >= position)
    //             {
    //                 self.move_selection_down();
    //             }
    //         }
    //         HistoryOperation::DeleteElement { tree_path, .. } => {
    //             self.delete_at(&tree_path);

    //             // if edit happened above selection, move the selection
    //             if tree_path
    //                 .get(self.parent_view_path.len())
    //                 .is_some_and(|&position| self.selection_index() >= position)
    //             {
    //                 self.move_selection_up();
    //             }
    //         }
    //     }
    // }

    // pub fn push_view_stack(&mut self) {
    //     self.parent_view_path.push(self.selection_index());
    //     self.update_selection(0);

    //     let parent_node = self.parent_node();
    //     if parent_node.children.is_empty() {
    //         self.insert_at(&self.path_to_selection(), ElementTree::new_leaf(NEW_ELEMENT_TEXT));
    //     }
    // }

    // pub fn pop_view_stack(&mut self) {
    //     if let Some(new_selection) = self.parent_view_path.pop() {
    //         self.update_selection(new_selection);
    //     }
    // }

    // pub fn parent_tree_stack(&self) -> Vec<&ElementTree> {
    //     if self.parent_view_path.is_empty() {
    //         vec![]
    //     } else {
    //         let mut current = &self.trees[self.parent_view_path[0]];
    //         let mut stack = vec![current];
    //         for &index in &self.parent_view_path[1..] {
    //             current = &current.children[index];
    //             stack.push(current);
    //         }
    //         stack
    //     }
    // }
}
