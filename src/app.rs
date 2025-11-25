use crate::io;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tui_textarea::TextArea;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Status {
    Todo,
    Doing,
    Done,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub title: String,
    //ensures compatibility with old JSON files (adds empty string if missing)
    #[serde(default)]
    pub description: String,
    pub status: Status,
}

pub struct App<'a> {
    pub tasks: Vec<Task>,
    pub active_column: usize,
    pub selected_index: usize,

    // Modes
    pub input_mode: bool,       // Simple line input (Title)
    pub input_buffer: String,   // Buffer for title
    pub cursor_position: usize, // Cursor for title
    pub is_editing: bool,       // Context for title input

    pub delete_mode: bool,    // Delete confirmation popup
    pub edit_desc_mode: bool, // Unified Description View/Edit mode

    pub file_path: PathBuf,

    pub description_editor: TextArea<'a>,
}

impl<'a> App<'a> {
    pub fn new(file_path: PathBuf) -> Self {
        let tasks = io::load_tasks(&file_path);

        let textarea = TextArea::default();

        App {
            tasks,
            active_column: 0,
            selected_index: 0,
            input_mode: false,
            input_buffer: String::new(),
            cursor_position: 0,
            is_editing: false,
            delete_mode: false,
            edit_desc_mode: false,
            file_path,
            description_editor: textarea,
        }
    }

    fn save(&self) {
        if let Err(e) = io::save_tasks(&self.file_path, &self.tasks) {
            eprintln!("Error saving tasks: {}", e);
        }
    }

    pub fn get_tasks_in_column(&self, col_idx: usize) -> Vec<&Task> {
        let status = match col_idx {
            0 => Status::Todo,
            1 => Status::Doing,
            _ => Status::Done,
        };
        self.tasks.iter().filter(|t| t.status == status).collect()
    }

    fn get_selected_global_index(&self) -> Option<usize> {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        let task_ref = tasks_in_col.get(self.selected_index)?;

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
        if self.selected_index > 0 && self.selected_index < tasks_in_col.len() {
            let current = tasks_in_col[self.selected_index];
            let target = tasks_in_col[self.selected_index - 1];
            if let (Some(c), Some(t)) = (self.find_idx(current), self.find_idx(target)) {
                self.tasks.swap(c, t);
                self.selected_index -= 1;
                self.save();
            }
        }
    }

    pub fn move_task_down(&mut self) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        if !tasks_in_col.is_empty() && self.selected_index < tasks_in_col.len() - 1 {
            let current = tasks_in_col[self.selected_index];
            let target = tasks_in_col[self.selected_index + 1];
            if let (Some(c), Some(t)) = (self.find_idx(current), self.find_idx(target)) {
                self.tasks.swap(c, t);
                self.selected_index += 1;
                self.save();
            }
        }
    }

    fn find_idx(&self, task: &Task) -> Option<usize> {
        self.tasks.iter().position(|t| std::ptr::eq(t, task))
    }

    // --- TITLE INPUT HANDLING ---

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let mut chars: Vec<char> = self.input_buffer.chars().collect();
        if self.cursor_position <= chars.len() {
            chars.insert(self.cursor_position, new_char);
            self.input_buffer = chars.into_iter().collect();
            self.move_cursor_right();
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.input_buffer.chars().collect();
            if self.cursor_position <= chars.len() {
                chars.remove(self.cursor_position - 1);
                self.input_buffer = chars.into_iter().collect();
                self.move_cursor_left();
            }
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_buffer.chars().count())
    }

    // --- TITLE ACTIONS ---

    pub fn start_adding(&mut self) {
        self.input_mode = true;
        self.is_editing = false;
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    pub fn start_editing(&mut self) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        if let Some(task) = tasks_in_col.get(self.selected_index) {
            self.input_buffer = task.title.clone();
            self.input_mode = true;
            self.is_editing = true;
            self.cursor_position = self.input_buffer.chars().count();
        }
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = false;
        self.is_editing = false;
        self.input_buffer.clear();
    }

    pub fn submit_input(&mut self) {
        if self.input_buffer.trim().is_empty() {
            self.cancel_input();
            return;
        }

        if self.is_editing {
            if let Some(idx) = self.get_selected_global_index() {
                self.tasks[idx].title = self.input_buffer.trim().to_string();
                self.save();
            }
        } else {
            self.tasks.push(Task {
                title: self.input_buffer.trim().to_string(),
                description: String::new(),
                status: Status::Todo,
            });
            self.save();
        }

        self.input_mode = false;
        self.is_editing = false;
        self.input_buffer.clear();
    }

    // --- DELETE ---

    pub fn prompt_delete(&mut self) {
        if !self.get_tasks_in_column(self.active_column).is_empty() {
            self.delete_mode = true;
        }
    }

    pub fn cancel_delete(&mut self) {
        self.delete_mode = false;
    }

    pub fn confirm_delete(&mut self) {
        if let Some(idx) = self.get_selected_global_index() {
            self.tasks.remove(idx);
            self.save();
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
        self.delete_mode = false;
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

    // --- UNIFIED DESCRIPTION EDITOR (VIEW + EDIT) ---

    /// Opens the unified editor with the current description
    pub fn open_description(&mut self) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        if let Some(task) = tasks_in_col.get(self.selected_index) {
            // Load existing description into TextArea (needs lines as Vec<String>)
            let lines: Vec<String> = task.description.lines().map(|s| s.to_string()).collect();
            self.description_editor = TextArea::new(lines);
            self.edit_desc_mode = true;
        }
    }

    /// Saves the content of the editor back to the task and closes it
    pub fn close_description(&mut self) {
        // Join lines back into a single string
        let new_desc = self.description_editor.lines().join("\n");

        if let Some(idx) = self.get_selected_global_index() {
            self.tasks[idx].description = new_desc;
            self.save();
        }
        self.edit_desc_mode = false;
    }
}
