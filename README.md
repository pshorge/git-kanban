# git-kanban 🦀📋

A lightweight, terminal-based (TUI) Kanban board for your git projects. 
It stores tasks locally within the `.git` directory, keeping your personal notes private and out of your remote repository history.

Built with **Rust** and **Ratatui**.

## 🚀 Features

- **Local & Private**: Tasks are saved in `.git/git-kanban.json`. They are specific to the repository but are **never** committed or pushed to the server.
- **Simple Workflow**: Three classic columns: `TODO` → `DOING` → `DONE`.
- **Keyboard Driven**: Fast navigation using arrows and shortcuts.
- **Blazing Fast**: Written in Rust, instant startup.

## 📦 Installation

### Prerequisites
- Rust and Cargo installed (via `rustup`).

### Build and Install
Clone the repository and install the binary using Cargo:

```bash
# Go to the project folder
cd git-kanban

# Install globally
cargo install --path .
