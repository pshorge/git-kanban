use crate::app::App;
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
        "Git Kanban (Project Mode)"
    } else {
        "Git Kanban (Standalone Mode)"
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

        let border_style =
            if app.active_column == i && !app.input_mode && !app.delete_mode && !app.edit_desc_mode
            {
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

    // 3. Footer
    if app.input_mode {
        let title = if app.is_editing {
            "Edit Title"
        } else {
            "New Task"
        };

        let block = Block::default().borders(Borders::ALL).title(title);
        let inner_area = block.inner(chunks[2]);

        let input = Paragraph::new(app.input_buffer.as_str())
            .style(Style::default().fg(Color::Green))
            .block(block);

        f.render_widget(input, chunks[2]);

        let cursor_x = inner_area.x + app.cursor_position as u16;
        let cursor_y = inner_area.y;

        f.set_cursor_position(Position::new(cursor_x, cursor_y));
    } else {
        let help_text = "q:Quit | n:New | e:Edit Title | v:Desc | d:Delete | Shift+↑/↓:Move";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    // 4. Delete Confirmation
    if app.delete_mode {
        let block = Block::default()
            .title("Confirmation")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red).bg(Color::Black));

        let text = vec![
            Line::from("Delete task?"),
            Line::from(""),
            Line::from("Y / N"),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        let area = centered_rect(30, 15, f.area());
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    // 5. UNIFIED DESCRIPTION EDITOR / VIEWER
    if app.edit_desc_mode {
        let area = centered_rect(80, 80, f.area());
        f.render_widget(Clear, area);

        let mut editor = app.description_editor.clone();

        editor.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description (Esc to Save & Close) ")
                .style(Style::default().fg(Color::Green).bg(Color::Black)),
        );

        f.render_widget(&editor, area);
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
