use std::{path::PathBuf, sync::LazyLock};

use color_eyre::Result;
use fs_err as fs;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::entities::{ParentTask, Task, TaskData};

mod view;
use view::ViewStorage;

static FILE_PATH: LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
    let mut path = PathBuf::from(std::env::var("HOME").expect("There is no $HOME"));
    path.push("nest_state.json");
    path
});

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppStorage {
    pub view: ViewStorage,
    pub tasks: IndexMap<u64, Task>,
}

impl AppStorage {
    pub fn insert_task(&mut self, parent: ParentTask, task_data: TaskData) {
        let mut task = self.create_task(task_data);
        task.parent = parent;

        if let ParentTask::Id(parent_id) = parent {
            self.tasks.entry(parent_id).or_default().children.push(task.id);
        }

        self.tasks.insert(task.id, task);
    }

    pub fn get_task(&self, task_id: u64) -> Option<&Task> {
        self.tasks.get(&task_id)
    }

    pub fn find_parents_stack(&self) -> Vec<&Task> {
        let mut parents = Vec::new();

        let mut current_id = match self.view.get_opened_task() {
            ParentTask::Id(id) => id,
            ParentTask::Root => return vec![],
        };

        while let Some(task) = self.tasks.get(&current_id) {
            parents.push(task);
            match task.parent {
                ParentTask::Id(parent_id) => current_id = parent_id,
                ParentTask::Root => break,
            }
        }
        parents
    }

    pub fn find_tasks_to_display(&self) -> Vec<&Task> {
        match self.view.get_opened_task() {
            ParentTask::Root => self.find_root_tasks(),
            ParentTask::Id(parent_id) => self.find_sub_tasks(parent_id),
        }
    }

    pub fn find_root_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.parent == ParentTask::Root)
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
        let parent = self.tasks.get(&task_id)?.parent;

        if let ParentTask::Id(parent_id) = parent {
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

    pub fn swap_sub_tasks(&mut self, from: u64, to: u64) -> Option<()> {
        let parent = self.view.get_opened_task();
        match parent {
            ParentTask::Id(parent_id) => {
                let parent_task = self.tasks.get_mut(&parent_id)?;
                let from_index = parent_task.children.iter().position(|id| *id == from)?;
                let to_index = parent_task.children.iter().position(|id| *id == to)?;
                parent_task.children.swap(from_index, to_index);
            }
            ParentTask::Root => {
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

    pub fn load_state() -> Result<AppStorage> {
        fs::read_to_string(&*FILE_PATH)
            .and_then(|json| serde_json::from_str(&json).map_err(|err| err.into()))
            .or_else(|_| Ok(AppStorage::default()))
    }
}
