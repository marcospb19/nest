use crate::tree::ElementTree;

#[derive(Debug, Clone)]
pub enum HistoryOperation {
    AddElement {
        text: String,
        tree_path: Vec<usize>,
    },
    DeleteElement {
        text: String,
        tree_path: Vec<usize>,
        children: Vec<ElementTree>,
    },
}

pub struct History {
    undo_stack: Vec<HistoryOperation>,
    redo_stack: Vec<HistoryOperation>,
}

impl History {
    pub fn new() -> Self {
        History {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push_operation(&mut self, operation: HistoryOperation) {
        self.undo_stack.push(operation);
        self.redo_stack.clear();
    }

    pub fn undo_operation(&mut self) -> Option<HistoryOperation> {
        match self.undo_stack.pop() {
            Some(operation) => {
                self.redo_stack.push(operation.clone());
                Some(operation)
            }
            None => None,
        }
    }

    pub fn redo_operation(&mut self) -> Option<HistoryOperation> {
        match self.redo_stack.pop() {
            Some(operation) => {
                self.undo_stack.push(operation.clone());
                Some(operation)
            }
            None => None,
        }
    }
}
