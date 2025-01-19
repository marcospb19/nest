//! Store and read the state from the disk.

use std::{fs, path::Path};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::App;

static FILE_PATH: &str = "state.json";

#[derive(Serialize, Deserialize)]
pub struct State {
    names: Vec<String>,
    selected_name: usize,
}

impl State {
    pub fn from_app(app: &App) -> Self {
        State {
            names: app.names.clone(),
            selected_name: app.list.selected().unwrap(),
        }
    }

    pub fn into_app(self) -> App {
        App::from_names(self.names)
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
