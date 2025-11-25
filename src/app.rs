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
    #[serde(default)]
    pub description: String,
    pub status: Status,
}

// Enum to track focus in the split edit window
#[derive(Debug, PartialEq)]
pub enum EditFocus {
    Title,
    Description,
}

pub struct App<'a> {
    pub tasks: Vec<Task>,
    pub active_column: usize,
    pub selected_index: usize,

    // --- MODES ---
    pub input_mode: bool,       // Footer Quick Add
    pub input_buffer: String,   // Footer Buffer
    pub cursor_position: usize, // Footer Cursor

    pub delete_mode: bool, // Confirmation popup

    pub view_mode: bool, // Read-only popup

    pub edit_mode: bool,           // Split Edit popup
    pub edit_focus: EditFocus,     // Which part is active?
    pub edit_title_buffer: String, // Temporary buffer for title editing
    pub edit_cursor_pos: usize,    // Cursor for title editing

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

            delete_mode: false,

            view_mode: false,

            edit_mode: false,
            edit_focus: EditFocus::Title,
            edit_title_buffer: String::new(),
            edit_cursor_pos: 0,

            file_path,
            description_editor: textarea,
        }
    }

    fn save(&self) {
        if let Err(e) = io::save_tasks(&self.file_path, &self.tasks) {
            eprintln!("Error saving tasks: {}", e);
        }
    }

    // --- HELPERS ---
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

    // --- NAVIGATION & REORDERING ---
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

    // --- FOOTER INPUT ---
    pub fn start_adding(&mut self) {
        self.input_mode = true;
        self.input_buffer.clear();
        self.cursor_position = 0;
    }
    pub fn cancel_input(&mut self) {
        self.input_mode = false;
    }
    pub fn submit_input(&mut self) {
        if self.input_buffer.trim().is_empty() {
            self.cancel_input();
            return;
        }
        self.tasks.push(Task {
            title: self.input_buffer.trim().to_string(),
            description: String::new(),
            status: Status::Todo,
        });
        self.save();
        self.input_mode = false;
    }
    // Manual cursor logic
    pub fn enter_char_footer(&mut self, c: char) {
        let mut chars: Vec<char> = self.input_buffer.chars().collect();
        if self.cursor_position <= chars.len() {
            chars.insert(self.cursor_position, c);
            self.input_buffer = chars.into_iter().collect();
            self.cursor_position += 1;
        }
    }
    pub fn delete_char_footer(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.input_buffer.chars().collect();
            if self.cursor_position <= chars.len() {
                chars.remove(self.cursor_position - 1);
                self.input_buffer = chars.into_iter().collect();
                self.cursor_position -= 1;
            }
        }
    }
    pub fn move_cursor_left_footer(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_right_footer(&mut self) {
        if self.cursor_position < self.input_buffer.chars().count() {
            self.cursor_position += 1;
        }
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

    // --- VIEW MODE (Read Only) ---
    pub fn open_view_mode(&mut self) {
        if self.get_tasks_in_column(self.active_column).is_empty() {
            return;
        }
        self.view_mode = true;
    }
    pub fn close_view_mode(&mut self) {
        self.view_mode = false;
    }
    pub fn get_current_task_info(&self) -> (String, String) {
        let tasks_in_col = self.get_tasks_in_column(self.active_column);
        if let Some(task) = tasks_in_col.get(self.selected_index) {
            (task.title.clone(), task.description.clone())
        } else {
            (String::new(), String::new())
        }
    }

    // --- EDIT MODE ---
    pub fn open_edit_mode(&mut self) {
        if let Some(idx) = self.get_selected_global_index() {
            // Load title into manual buffer
            let title = self.tasks[idx].title.clone();
            let description = self.tasks[idx].description.clone();

            self.edit_title_buffer = title;
            self.edit_cursor_pos = self.edit_title_buffer.chars().count();

            // Load description into TextArea
            let lines: Vec<String> = description.lines().map(|s| s.to_string()).collect();
            self.description_editor = TextArea::new(lines);

            self.edit_focus = EditFocus::Title;
            self.edit_mode = true;
        }
    }
    pub fn close_edit_mode(&mut self) {
        self.edit_mode = false;
    }
    pub fn toggle_edit_focus(&mut self) {
        self.edit_focus = match self.edit_focus {
            EditFocus::Title => EditFocus::Description,
            EditFocus::Description => EditFocus::Title,
        };
    }
    pub fn save_edit_changes(&mut self) {
        if self.edit_title_buffer.trim().is_empty() {
            return;
        } // Prevent empty title
        let new_desc = self.description_editor.lines().join("\n");
        if let Some(idx) = self.get_selected_global_index() {
            self.tasks[idx].title = self.edit_title_buffer.trim().to_string();
            self.tasks[idx].description = new_desc;
            self.save();
        }
        self.edit_mode = false;
    }

    // Manual Title Editing logic
    pub fn enter_char_edit_title(&mut self, c: char) {
        let mut chars: Vec<char> = self.edit_title_buffer.chars().collect();
        if self.edit_cursor_pos <= chars.len() {
            chars.insert(self.edit_cursor_pos, c);
            self.edit_title_buffer = chars.into_iter().collect();
            self.edit_cursor_pos += 1;
        }
    }
    pub fn delete_char_edit_title(&mut self) {
        if self.edit_cursor_pos > 0 {
            let mut chars: Vec<char> = self.edit_title_buffer.chars().collect();
            if self.edit_cursor_pos <= chars.len() {
                chars.remove(self.edit_cursor_pos - 1);
                self.edit_title_buffer = chars.into_iter().collect();
                self.edit_cursor_pos -= 1;
            }
        }
    }
}
