//! Store and read the state from the disk.

use std::path::Path;

use color_eyre::Result;
use fs_err as fs;
use serde::{Deserialize, Serialize};

use crate::{app::App, tree::ElementTree};

static FILE_PATH: &str = "state.json";

#[derive(Serialize, Deserialize)]
pub struct State {
    elements: Vec<ElementTree>,
    selection_path: Vec<usize>,
}

impl State {
    pub fn from_app(app: &App) -> Self {
        let mut selection_path = app.parent_view_path.clone();
        selection_path.push(app.selection_index());

        State {
            selection_path,
            elements: app.trees.clone(),
        }
    }

    pub fn into_app(self) -> App {
        App::from_trees(self.elements)
    }
}

pub fn save_state(state: &State) -> Result<()> {
    let json = serde_json::to_string(state)?;
    fs::write(FILE_PATH, json)?;
    Ok(())
}

pub fn load_state() -> Result<Option<State>> {
    if Path::new(FILE_PATH).try_exists()? {
        let json = fs::read_to_string(FILE_PATH)?;
        let state = serde_json::from_str(&json)?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}
