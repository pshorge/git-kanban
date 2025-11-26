use crate::app::{App, EditFocus};
use ratatui::{prelude::*, widgets::*};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    // 1. Header
    let path_str = app.file_path.to_string_lossy();
    let title_text = if path_str.contains(".git") {
        "Git Kanban (Project)"
    } else {
        "Git Kanban (Local)"
    };
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // 2. Columns
    let columns_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);
    let column_titles = ["TODO", "DOING", "DONE"];
    for i in 0..3 {
        let tasks = app.get_tasks_in_column(i);
        let items: Vec<ListItem> = tasks
            .iter()
            .map(|t| ListItem::new(format!("• {}", t.title)))
            .collect();
        let is_modal = app.input_mode || app.delete_mode || app.view_mode || app.edit_mode;
        let border_style = if app.active_column == i && !is_modal {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(column_titles[i])
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
        if app.active_column == i {
            let mut state = ListState::default();
            state.select(Some(app.selected_index));
            f.render_stateful_widget(list, columns_layout[i], &mut state);
        } else {
            f.render_widget(list, columns_layout[i]);
        }
    }

    // 3. Footer (Quick Add)
    if app.input_mode {
        let mut editor = app.title_editor.clone();
        editor.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" New Task (Enter to Save) ")
                .style(Style::default().fg(Color::Green)),
        );
        f.render_widget(&editor, chunks[2]);
    } else {
        let help_text = "q:Quit | n:New | e:Edit | v:View | d:Delete | Shift+↑/↓:Move";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    // 4. VIEW MODE
    if app.view_mode {
        let area = centered_rect(60, 60, f.area());
        f.render_widget(Clear, area);
        let block = Block::default()
            .title(" Task Details (Esc to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let inner = block.inner(area);
        f.render_widget(block, area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1)])
            .split(inner);
        let (title_str, desc_str) = app.get_current_task_info();
        let title_p = Paragraph::new(title_str).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        );
        f.render_widget(title_p, layout[0]);
        let desc_text = if desc_str.is_empty() {
            "(No description)"
        } else {
            &desc_str
        };
        let desc_p = Paragraph::new(desc_text).wrap(Wrap { trim: false });
        let divider = Block::default().borders(Borders::TOP);
        f.render_widget(divider, layout[1]);
        let desc_area = Layout::default()
            .constraints([Constraint::Min(1)])
            .margin(1)
            .split(layout[1])[0];
        f.render_widget(desc_p, desc_area);
    }

    // 5. EDIT MODE
    if app.edit_mode {
        let area = centered_rect(80, 80, f.area());
        f.render_widget(Clear, area);
        let main_block = Block::default()
            .title(" Edit Task (Tab: Switch | Ctrl+S: Save | Esc: Cancel) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let inner = main_block.inner(area);
        f.render_widget(main_block, area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(inner);

        // Title Editor
        let title_color = if app.edit_focus == EditFocus::Title {
            Color::Green
        } else {
            Color::White
        };
        let mut t_editor = app.title_editor.clone();
        t_editor.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Title ")
                .style(Style::default().fg(title_color)),
        );
        f.render_widget(&t_editor, layout[0]);

        // Description Editor
        let desc_color = if app.edit_focus == EditFocus::Description {
            Color::Green
        } else {
            Color::White
        };
        let mut d_editor = app.description_editor.clone();
        d_editor.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description ")
                .style(Style::default().fg(desc_color)),
        );
        f.render_widget(&d_editor, layout[1]);
    }

    // 6. Delete Confirmation
    if app.delete_mode {
        let area = centered_rect(30, 15, f.area());
        f.render_widget(Clear, area);
        let block = Block::default()
            .title("Confirm")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red).bg(Color::Black));
        let text = vec![
            Line::from("Delete task?"),
            Line::from(""),
            Line::from("Y / N"),
        ];
        let p = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(p, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
