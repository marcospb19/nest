use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementTree {
    pub children: Vec<ElementTree>,
    pub text: String,
}

impl ElementTree {
    pub fn new_leaf(text: impl ToString) -> Self {
        ElementTree {
            children: Vec::new(),
            text: text.to_string(),
        }
    }

    pub fn new_branch(text: impl ToString, children: Vec<ElementTree>) -> Self {
        ElementTree {
            children,
            text: text.to_string(),
        }
    }

    // panics if tree_path is empty
    // panics if vec.get(index) returns None
    // #[track_caller]
    pub fn remove_node(&mut self, node_path: &[usize]) -> Self {
        match node_path {
            [] => panic!("tree_path is empty and refers to no element"),
            [last_index] => self.children.remove(*last_index),
            [current_index, path_rest @ ..] => {
                self.children
                    .get_mut(*current_index)
                    .expect("index out of bounds")
                    .remove_node(path_rest) // recursive call
            }
        }
    }

    #[track_caller]
    pub fn insert_node(&mut self, node_path: &[usize], new_node: ElementTree) {
        match node_path {
            [] => panic!("tree_path is empty and refers to no element"),
            [last_index] => self.children.insert(*last_index, new_node),
            [current_index, path_rest @ ..] => {
                self.children
                    .get_mut(*current_index)
                    .expect("index out of bounds")
                    .insert_node(path_rest, new_node) // recursive call
            }
        }
    }

    pub fn get_node(&self, node_path: &[usize]) -> Option<&ElementTree> {
        match node_path {
            [] => panic!("tree_path is empty and refers to no element"),
            [last_index] => self.children.get(*last_index),
            [current_index, path_rest @ ..] => {
                self.children
                    .get(*current_index)
                    .expect("index out of bounds")
                    .get_node(path_rest) // recursive call
            }
        }
    }
}
