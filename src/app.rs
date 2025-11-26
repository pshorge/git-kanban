use crate::io;
use ratatui::style::Style;
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

#[derive(Debug, PartialEq)]
pub enum EditFocus {
    Title,
    Description,
}

pub struct App<'a> {
    pub tasks: Vec<Task>,
    pub active_column: usize,
    pub selected_index: usize,

    // Modes
    pub input_mode: bool,  // Footer Quick Add
    pub view_mode: bool,   // Read-only Modal
    pub delete_mode: bool, // Delete Confirm
    pub edit_mode: bool,   // Split Edit Modal

    pub edit_focus: EditFocus, // Which box is active in edit mode?

    pub file_path: PathBuf,

    // EDITORS
    pub title_editor: TextArea<'a>,
    pub description_editor: TextArea<'a>,
}

impl<'a> App<'a> {
    pub fn new(file_path: PathBuf) -> Self {
        let tasks = io::load_tasks(&file_path);

        let mut title_ta = TextArea::default();
        title_ta.set_cursor_line_style(Style::default());

        let desc_ta = TextArea::default();

        App {
            tasks,
            active_column: 0,
            selected_index: 0,

            input_mode: false,
            view_mode: false,
            delete_mode: false,
            edit_mode: false,
            edit_focus: EditFocus::Title,

            file_path,

            title_editor: title_ta,
            description_editor: desc_ta,
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

    // --- FOOTER INPUT (Quick Add) ---
    pub fn start_adding(&mut self) {
        self.input_mode = true;
        self.title_editor = TextArea::default();
        self.title_editor.set_cursor_line_style(Style::default()); // Single line feel
    }
    pub fn cancel_input(&mut self) {
        self.input_mode = false;
    }

    pub fn submit_input(&mut self) {
        // Join lines to ensure single line title
        let title = self.title_editor.lines().join(" ");

        if title.trim().is_empty() {
            self.cancel_input();
            return;
        }

        self.tasks.push(Task {
            title: title.trim().to_string(),
            description: String::new(),
            status: Status::Todo,
        });
        self.save();
        self.input_mode = false;
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

    // --- VIEW MODE ---
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

    // --- EDIT MODE (Split Window) ---
    pub fn open_edit_mode(&mut self) {
        // Use global index to avoid borrow checker issues later
        if let Some(idx) = self.get_selected_global_index() {
            let title = self.tasks[idx].title.clone();
            let description = self.tasks[idx].description.clone();

            // Load Title into TextArea
            self.title_editor = TextArea::new(vec![title]);
            self.title_editor.set_cursor_line_style(Style::default());
            self.title_editor.move_cursor(tui_textarea::CursorMove::End);

            // Load Description
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
        let new_title = self.title_editor.lines().join(" ");
        if new_title.trim().is_empty() {
            return;
        }

        let new_desc = self.description_editor.lines().join("\n");

        if let Some(idx) = self.get_selected_global_index() {
            self.tasks[idx].title = new_title.trim().to_string();
            self.tasks[idx].description = new_desc;
            self.save();
        }
        self.edit_mode = false;
    }
}
