use crate::io;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Status {
    Todo,
    Doing,
    Done,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub title: String,
    pub status: Status,
}

pub struct App {
    pub tasks: Vec<Task>,
    pub active_column: usize,
    pub selected_index: usize,

    // Input related fields
    pub input_mode: bool,
    pub input_buffer: String,
    pub is_editing: bool,

    file_path: PathBuf,
}

impl App {
    pub fn new(file_path: PathBuf) -> Self {
        let tasks = io::load_tasks(&file_path);
        App {
            tasks,
            active_column: 0,
            selected_index: 0,
            input_mode: false,
            input_buffer: String::new(),
            is_editing: false,
            file_path,
        }
    }

    /// Helper: Saves state to disk
    fn save(&self) {
        if let Err(e) = io::save_tasks(&self.file_path, &self.tasks) {
            eprintln!("Error saving tasks: {}", e);
        }
    }

    /// Helper: Gets tasks strictly for the current column view
    pub fn get_tasks_in_column(&self, col_idx: usize) -> Vec<&Task> {
        let status = match col_idx {
            0 => Status::Todo,
            1 => Status::Doing,
            _ => Status::Done,
        };
        self.tasks.iter().filter(|t| t.status == status).collect()
    }

    /// Helper: Finds the real index in self.tasks based on visual selection
    fn get_selected_global_index(&self) -> Option<usize> {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        let task_ref = tasks_in_col.get(self.selected_index)?;

        // Find this task in the main vector.
        // In a prod app, use UUIDs. Here we match title+status (simplified).
        self.tasks
            .iter()
            .position(|t| t.title == task_ref.title && t.status == task_ref.status)
    }

    // --- NAVIGATION ---

    pub fn next_column(&mut self) {
        if self.active_column < 2 {
            self.active_column += 1;
            self.selected_index = 0;
        }
    }

    pub fn prev_column(&mut self) {
        if self.active_column > 0 {
            self.active_column -= 1;
            self.selected_index = 0;
        }
    }

    pub fn next_item(&mut self) {
        let count = self.get_tasks_in_column(self.active_column).len();
        if count > 0 && self.selected_index < count - 1 {
            self.selected_index += 1;
        }
    }

    pub fn prev_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    // --- REORDERING ---

    pub fn move_task_up(&mut self) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);

        // We can only move up if we are not at the top (index > 0)
        if self.selected_index > 0 && self.selected_index < tasks_in_col.len() {
            let current_task_ref = tasks_in_col[self.selected_index];
            let target_task_ref = tasks_in_col[self.selected_index - 1];

            // Find global indices
            let current_global_idx = self
                .tasks
                .iter()
                .position(|t| std::ptr::eq(t, current_task_ref));
            let target_global_idx = self
                .tasks
                .iter()
                .position(|t| std::ptr::eq(t, target_task_ref));

            if let (Some(curr), Some(target)) = (current_global_idx, target_global_idx) {
                self.tasks.swap(curr, target);
                self.selected_index -= 1; // Follow the item visually
                self.save();
            }
        }
    }

    pub fn move_task_down(&mut self) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);

        // We can only move down if we are not at the bottom
        if !tasks_in_col.is_empty() && self.selected_index < tasks_in_col.len() - 1 {
            let current_task_ref = tasks_in_col[self.selected_index];
            let target_task_ref = tasks_in_col[self.selected_index + 1];

            // Find global indices
            let current_global_idx = self
                .tasks
                .iter()
                .position(|t| std::ptr::eq(t, current_task_ref));
            let target_global_idx = self
                .tasks
                .iter()
                .position(|t| std::ptr::eq(t, target_task_ref));

            if let (Some(curr), Some(target)) = (current_global_idx, target_global_idx) {
                self.tasks.swap(curr, target);
                self.selected_index += 1; // Follow the item visually
                self.save();
            }
        }
    }

    // --- ACTIONS ---

    /// Enter input mode to create a NEW task
    pub fn start_adding(&mut self) {
        self.input_mode = true;
        self.is_editing = false;
        self.input_buffer.clear();
    }

    /// Enter input mode to EDIT the selected task
    pub fn start_editing(&mut self) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        if let Some(task) = tasks_in_col.get(self.selected_index) {
            self.input_buffer = task.title.clone();
            self.input_mode = true;
            self.is_editing = true;
        }
    }

    /// Cancels input mode without saving
    pub fn cancel_input(&mut self) {
        self.input_mode = false;
        self.is_editing = false;
        self.input_buffer.clear();
    }

    /// Finalizes input (Add New or Update Existing)
    pub fn submit_input(&mut self) {
        if self.input_buffer.trim().is_empty() {
            self.cancel_input();
            return;
        }

        if self.is_editing {
            // UPDATE EXISTING
            if let Some(idx) = self.get_selected_global_index() {
                self.tasks[idx].title = self.input_buffer.trim().to_string();
                self.save();
            }
        } else {
            // ADD NEW
            self.tasks.push(Task {
                title: self.input_buffer.trim().to_string(),
                status: Status::Todo,
            });
            self.save();
        }

        self.input_mode = false;
        self.is_editing = false;
        self.input_buffer.clear();
    }

    pub fn delete_current_task(&mut self) {
        if let Some(idx) = self.get_selected_global_index() {
            self.tasks.remove(idx);
            self.save();
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }

    pub fn move_current_task(&mut self) {
        if let Some(idx) = self.get_selected_global_index() {
            let task = &mut self.tasks[idx];
            task.status = match task.status {
                Status::Todo => Status::Doing,
                Status::Doing => Status::Done,
                Status::Done => Status::Todo,
            };
            self.save();
        }
    }
}
