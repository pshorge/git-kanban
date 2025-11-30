# git-kanban 🦀📋

A lightweight, terminal-based (TUI) Kanban board built with **Rust**. 

It is designed to keep your tasks private and local. It works seamlessly inside Git repositories (storing data in `.git/`) or as a standalone task manager in any directory.

![Git Kanban Demo](assets/demo1.png)

![Git Kanban Demo](assets/demo2.png)

![Git Kanban Demo](assets/demo3.png)

## 🚀 Features

- **🔒 Local & Private**:
  - **Project Mode**: If run inside a git repo, tasks are saved in `.git/git-kanban.json` (not committed to history).
  - **Standalone Mode**: If run elsewhere, tasks are saved in `.kanban.json` (hidden file).
- **📝 Advanced Editing**: Split-window editor for Title and Description using `tui-textarea`.
- **✏️ Full CRUD**: Create, Read, Update, and Delete tasks.
- **↕️ Reordering**: Move tasks up and down within a column using `Shift + ↑/↓`.
- **🛡️ Safety First**: Confirmation modal before deleting tasks.
- **✨ Better UX**: Visual cursor support in all input fields.
- **⚡ Blazing Fast**: Written in Rust using `ratatui`.

## 📦 Installation

### Prerequisites
- Rust and Cargo installed.

### Build and Install
Clone the repository and install the binary:

```bash
git clone https://github.com/pshorge/git-kanban.git
cd git-kanban
cargo install --path . --force
```

## ⚙️ Setup as Git Alias (Optional)
To use it as a native git command:

```bash
git config --global alias.kanban '!git-kanban'
git kanban
```

