use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TaskData {
    pub title: String,
    pub children: Vec<u64>,
    pub done: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskEntity {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub title: String,
    pub children: Vec<u64>,
    pub done: bool,
}

impl TaskEntity {
    pub fn with_data(self, data: TaskData) -> Self {
        TaskEntity {
            id: self.id,
            parent_id: self.parent_id,
            title: data.title,
            children: data.children,
            done: data.done,
        }
    }
}

impl From<TaskEntity> for TaskData {
    fn from(data: TaskEntity) -> Self {
        TaskData {
            title: data.title,
            children: data.children,
            done: data.done,
        }
    }
}
