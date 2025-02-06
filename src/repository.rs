use std::{
    collections::HashMap,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use color_eyre::Result;
use fs_err as fs;
use serde::{Deserialize, Serialize};

use crate::entities::{TaskData, TaskEntity};

static FILE_PATH: &str = "state.json";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppTreeRepository {
    pub tasks: HashMap<u64, TaskEntity>,
}

impl AppTreeRepository {
    pub fn insert_task(&mut self, task_data: TaskData) {
        let task_entity = Self::create_task_entity(task_data);
        self.tasks.insert(task_entity.id, task_entity);
    }

    pub fn insert_sub_task(&mut self, parent_id: u64, task_data: TaskData) {
        let mut task_entity = Self::create_task_entity(task_data);
        task_entity.parent_id = Some(parent_id);

        self.tasks.insert(task_entity.id, task_entity.clone());
        self.tasks.entry(parent_id).or_default().children.push(task_entity.id);
    }

    pub fn get_task(&self, task_id: u64) -> Option<&TaskEntity> {
        self.tasks.get(&task_id)
    }

    pub fn find_parents_stack(&self, task_id: u64) -> Vec<&TaskEntity> {
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

    pub fn find_root_tasks(&self) -> Vec<&TaskEntity> {
        self.tasks
            .values()
            .filter(|task| task.parent_id.is_none())
            .collect::<Vec<_>>()
    }

    pub fn find_sub_tasks(&self, parent_id: u64) -> Vec<&TaskEntity> {
        match self.tasks.get(&parent_id) {
            None => return vec![],
            Some(parent_task) => parent_task
                .children
                .iter()
                .filter_map(|id| self.tasks.get(id))
                .collect(),
        }
    }

    pub fn remove_task(&mut self, task_id: u64) -> Option<TaskEntity> {
        let parent_id = self.tasks.get(&task_id).and_then(|task| task.parent_id);

        if let Some(parent_id) = parent_id {
            self.tasks
                .entry(parent_id)
                .or_default()
                .children
                .retain(|id| *id != task_id);
        }

        self.tasks.remove(&task_id)
    }

    pub fn update_task_title(&mut self, task_id: u64, new_title: String) {
        self.tasks.entry(task_id).and_modify(|task| task.title = new_title);
    }

    fn create_task_entity(task_data: TaskData) -> TaskEntity {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        let mut task = TaskEntity::default().with_data(task_data);
        task.id = timestamp;

        task
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(FILE_PATH, json)?;
        Ok(())
    }

    pub fn load_state() -> Result<AppTreeRepository> {
        if let Ok(json) = fs::read_to_string(FILE_PATH) {
            if let Ok(state) = serde_json::from_str(&json) {
                return Ok(state);
            }
        }

        let default_state = AppTreeRepository::default();
        let default_json = serde_json::to_string_pretty(&default_state)?;
        let mut file = fs::File::create(FILE_PATH)?;
        file.write_all(default_json.as_bytes())?;

        Ok(default_state)
    }
}
