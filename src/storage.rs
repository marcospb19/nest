use std::{path::PathBuf, sync::LazyLock};

use color_eyre::Result;
use fs_err as fs;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::entities::{Task, TaskData};

static FILE_PATH: LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
    let mut path = PathBuf::from(std::env::var("HOME").expect("There is no $HOME"));
    path.push("nest_state.json");
    path
});

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppTreeStorage {
    pub tasks: IndexMap<u64, Task>,
}

impl AppTreeStorage {
    pub fn insert_task(&mut self, task_data: TaskData) {
        let task = self.create_task(task_data);
        self.tasks.insert(task.id, task);
    }

    pub fn insert_sub_task(&mut self, parent_id: u64, task_data: TaskData) {
        let mut task = self.create_task(task_data);
        task.parent_id = Some(parent_id);

        self.tasks.insert(task.id, task.clone());
        self.tasks.entry(parent_id).or_default().children.push(task.id);
    }

    pub fn get_task(&self, task_id: u64) -> Option<&Task> {
        self.tasks.get(&task_id)
    }

    pub fn find_parents_stack(&self, task_id: u64) -> Vec<&Task> {
        let mut parents = Vec::new();
        let mut current_id = task_id;
        while let Some(task) = self.tasks.get(&current_id) {
            parents.push(task);
            match task.parent_id {
                None => break,
                Some(parent_id) => current_id = parent_id,
            }
        }
        parents
    }

    pub fn find_root_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.parent_id.is_none())
            .collect::<Vec<_>>()
    }

    pub fn find_sub_tasks(&self, parent_id: u64) -> Vec<&Task> {
        match self.tasks.get(&parent_id) {
            None => vec![],
            Some(parent_task) => parent_task
                .children
                .iter()
                .filter_map(|id| self.tasks.get(id))
                .collect(),
        }
    }

    pub fn remove_task(&mut self, task_id: u64) -> Option<Task> {
        let parent_id = self.tasks.get(&task_id).and_then(|task| task.parent_id);
        if let Some(parent_id) = parent_id {
            self.tasks
                .entry(parent_id)
                .or_default()
                .children
                .retain(|id| *id != task_id);
        }

        self.tasks.shift_remove(&task_id)
    }

    pub fn update_task_title(&mut self, task_id: u64, new_title: String) {
        self.tasks.entry(task_id).and_modify(|task| task.title = new_title);
    }

    pub fn update_task_state(&mut self, task_id: u64, done: bool) {
        self.tasks.entry(task_id).and_modify(|task| task.done = done);
    }

    pub fn swap_sub_tasks(&mut self, parent_id: Option<u64>, from: u64, to: u64) -> Option<()> {
        match parent_id {
            Some(parent_id) => {
                let parent_task = self.tasks.get_mut(&parent_id)?;
                let from_index = parent_task.children.iter().position(|id| *id == from)?;
                let to_index = parent_task.children.iter().position(|id| *id == to)?;
                parent_task.children.swap(from_index, to_index);
            }
            None => {
                let from_index = self.tasks.get_index_of(&from)?;
                let to_index = self.tasks.get_index_of(&to)?;
                self.tasks.swap_indices(from_index, to_index);
            }
        }
        Some(())
    }

    fn create_task(&self, task_data: TaskData) -> Task {
        let mut task = Task::default().with_data(task_data);

        let biggest_id = self.tasks.keys().max().unwrap_or(&0);
        task.id = biggest_id + 1;

        task
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&*FILE_PATH, json)?;
        Ok(())
    }

    pub fn load_state() -> Result<AppTreeStorage> {
        fs::read_to_string(&*FILE_PATH)
            .and_then(|json| serde_json::from_str(&json).map_err(|err| err.into()))
            .or_else(|_| Ok(AppTreeStorage::default()))
    }
}
