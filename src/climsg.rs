//! Integration with the `climsg` CLI tool.

use std::process::Command;

use crate::entities::Task;

pub fn send_message(task: Option<&Task>) {
    let msg = task
        .as_ref()
        .map_or_else(String::new, |task| format!(" {} ", task.title));
    Command::new("climsg")
        .arg("send")
        .arg("nest-current-task")
        .arg(msg)
        .status()
        .expect("Failed to send message");
}
