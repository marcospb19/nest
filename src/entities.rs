use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TaskData {
    pub title: String,
    pub children: Vec<u64>,
    pub done: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    // pub parent_id: Option<u64>,
    pub parent: ParentTask,
    pub title: String,
    pub children: Vec<u64>,
    pub done: bool,
}

impl Task {
    pub fn with_data(self, data: TaskData) -> Self {
        Task {
            id: self.id,
            parent: self.parent,
            title: data.title,
            children: data.children,
            done: data.done,
        }
    }
}

impl From<Task> for TaskData {
    fn from(data: Task) -> Self {
        TaskData {
            title: data.title,
            children: data.children,
            done: data.done,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParentTask {
    #[default]
    Root,
    Id(u64),
}
