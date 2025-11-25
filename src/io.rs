use crate::app::Task;
use anyhow::Result;
use std::{fs, path::PathBuf};

/// Determines where to save data.
pub fn find_storage_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let mut search_dir = current_dir.clone();

    // 1. Try to find .git upstream
    loop {
        let git_path = search_dir.join(".git");
        if git_path.exists() && git_path.is_dir() {
            return Ok(git_path.join("git-kanban.json"));
        }
        if !search_dir.pop() {
            break;
        }
    }

    // 2. Fallback
    Ok(current_dir.join(".kanban.json"))
}

pub fn load_tasks(file_path: &PathBuf) -> Vec<Task> {
    if file_path.exists() {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

pub fn save_tasks(file_path: &PathBuf, tasks: &[Task]) -> Result<()> {
    let json = serde_json::to_string_pretty(tasks)?;
    fs::write(file_path, json)?;
    Ok(())
}
