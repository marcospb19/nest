use ratatui::widgets::ListState;

use crate::{
    history::{History, HistoryOperation},
    tree::ElementTree,
};

const PLACEHOLDER_TEXT: &str = "Placeholder text.";
const NEW_ELEMENT_TEXT: &str = "New element text.";

pub struct App {
    pub parent_view_path: Vec<usize>,
    pub trees: Vec<ElementTree>,
    pub elements_list: ListState,
    pub stack_list: ListState,
    history: History,
}

impl App {
    pub fn from_trees(trees: Vec<ElementTree>) -> Self {
        let mut list = ListState::default();
        list.select(Some(0));

        Self {
            stack_list: ListState::default(),
            elements_list: list,
            trees,
            history: History::new(),
            parent_view_path: Vec::new(),
        }
    }

    pub fn new() -> Self {
        let trees = vec![
            ElementTree::new_leaf("Alice"),
            ElementTree::new_leaf("Bob"),
            ElementTree::new_leaf("Carol"),
            ElementTree::new_leaf("Dave"),
            ElementTree::new_leaf("Eve"),
        ];

        Self::from_trees(trees)
    }

    pub fn nodes_in_view(&self) -> &[ElementTree] {
        match self.parent_view_path.len() {
            0 => &self.trees,
            1 => &self.trees[self.parent_view_path[0]].children,
            _ => {
                &self.trees[self.parent_view_path[0]]
                    .get_node(&self.parent_view_path[1..])
                    .expect("parent_view_path must be valid")
                    .children
            }
        }
    }

    pub fn parent_node(&self) -> &ElementTree {
        let path = &self.parent_view_path;
        if path.len() == 1 {
            &self.trees[path[0]]
        } else {
            self.trees[path[0]]
                .get_node(&path[1..])
                .expect("parent_view_path must be valid")
        }
    }

    pub fn selection_index(&self) -> usize {
        self.elements_list.selected().unwrap()
    }

    #[expect(unused)]
    fn selected_node(&self) -> &ElementTree {
        &self.nodes_in_view()[self.selection_index()]
    }

    pub fn update_selection(&mut self, index: usize) {
        self.elements_list.select(Some(index));
    }

    fn path_to_selection(&self) -> Vec<usize> {
        self.parent_view_path
            .iter()
            .copied()
            .chain([self.selection_index()])
            .collect()
    }

    pub fn move_selection_up(&mut self) {
        let next = if self.selection_index() == 0 {
            self.trees.len() - 1 // Wrap to end
        } else {
            self.selection_index() - 1
        };
        self.update_selection(next);
    }

    pub fn move_selection_down(&mut self) {
        let next = if self.selection_index() + 1 >= self.trees.len() {
            0 // Wrap to start
        } else {
            self.selection_index() + 1
        };
        self.update_selection(next);
    }

    pub fn add_element_below(&mut self) {
        self.move_selection_down();
        let tree_path = self.path_to_selection();
        self.insert_at(&tree_path, ElementTree::new_leaf(PLACEHOLDER_TEXT));
        self.history.push_operation(HistoryOperation::AddElement {
            text: PLACEHOLDER_TEXT.to_string(),
            tree_path,
        });
    }

    pub fn delete_element(&mut self) {
        if self.trees.len() == 1 {
            if self.trees[0] == ElementTree::new_leaf(PLACEHOLDER_TEXT) {
                return;
            }
            self.add_element_below();
            self.move_selection_up();
        }

        let tree_path = self.path_to_selection();
        let deleted_node = self.delete_at(&tree_path);
        let deletion = HistoryOperation::DeleteElement {
            text: deleted_node.text,
            tree_path,
            children: deleted_node.children,
        };

        self.history.push_operation(deletion);
    }

    fn delete_at(&mut self, tree_path: &[usize]) -> ElementTree {
        if tree_path.len() == 1 {
            self.trees.remove(tree_path[0])
        } else {
            self.trees[tree_path[0]].remove_node(&tree_path[1..])
        }
    }

    fn insert_at(&mut self, tree_path: &[usize], new_leaf: ElementTree) {
        if tree_path.len() == 1 {
            self.trees.insert(tree_path[0], new_leaf);
        } else {
            self.trees[tree_path[0]].insert_node(&tree_path[1..], new_leaf);
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.update_selection(0);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.update_selection(self.trees.len() - 1);
    }

    pub fn undo_change(&mut self) {
        let Some(operation) = self.history.undo_operation() else {
            return;
        };
        match operation {
            HistoryOperation::AddElement { tree_path, .. } => {
                self.delete_at(&tree_path);

                // if edit happened above selection, move the selection
                if tree_path
                    .get(self.parent_view_path.len())
                    .is_some_and(|&position| self.selection_index() >= position)
                {
                    self.move_selection_up();
                }
            }
            HistoryOperation::DeleteElement {
                text,
                tree_path,
                children,
            } => {
                self.insert_at(&tree_path, ElementTree::new_branch(text, children));

                // if edit happened above selection, move the selection
                if tree_path
                    .get(self.parent_view_path.len())
                    .is_some_and(|&position| self.selection_index() >= position)
                {
                    self.move_selection_down();
                }
            }
        }
    }

    pub fn redo_change(&mut self) {
        let Some(operation) = self.history.redo_operation() else {
            return;
        };
        match operation {
            HistoryOperation::AddElement { text, tree_path } => {
                self.insert_at(&tree_path, ElementTree::new_branch(text, vec![]));

                // if edit happened above selection, move the selection
                if tree_path
                    .get(self.parent_view_path.len())
                    .is_some_and(|&position| self.selection_index() >= position)
                {
                    self.move_selection_down();
                }
            }
            HistoryOperation::DeleteElement { tree_path, .. } => {
                self.delete_at(&tree_path);

                // if edit happened above selection, move the selection
                if tree_path
                    .get(self.parent_view_path.len())
                    .is_some_and(|&position| self.selection_index() >= position)
                {
                    self.move_selection_up();
                }
            }
        }
    }

    pub fn push_view_stack(&mut self) {
        self.parent_view_path.push(self.selection_index());
        self.update_selection(0);

        let parent_node = self.parent_node();
        if parent_node.children.is_empty() {
            self.insert_at(&self.path_to_selection(), ElementTree::new_leaf(NEW_ELEMENT_TEXT));
        }
    }

    pub fn pop_view_stack(&mut self) {
        if let Some(new_selection) = self.parent_view_path.pop() {
            self.update_selection(new_selection);
        }
    }

    pub fn parent_tree_stack(&self) -> Vec<&ElementTree> {
        if self.parent_view_path.is_empty() {
            vec![]
        } else {
            let mut current = &self.trees[self.parent_view_path[0]];
            let mut stack = vec![current];
            for &index in &self.parent_view_path[1..] {
                current = &current.children[index];
                stack.push(current);
            }
            stack
        }
    }
}
