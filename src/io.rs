use anyhow::{Context, Result};
use std::{fs, path::PathBuf};
use crate::app::Task;

/// Finds the .git directory in current or parent directories
pub fn find_git_dir() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir()?;
    loop {
        let git_path = current_dir.join(".git");
        if git_path.exists() && git_path.is_dir() {
            return Ok(git_path);
        }
        if !current_dir.pop() {
            anyhow::bail!("Not inside a git repository!");
        }
    }
}

/// Loads tasks from the JSON file
pub fn load_tasks(file_path: &PathBuf) -> Vec<Task> {
    if file_path.exists() {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

/// Saves tasks to the JSON file
pub fn save_tasks(file_path: &PathBuf, tasks: &[Task]) -> Result<()> {
    let json = serde_json::to_string_pretty(tasks)?;
    fs::write(file_path, json).context("Failed to write kanban file")?;
    Ok(())
}