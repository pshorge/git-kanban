use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App) {
    // Layout: Header, Columns, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    // 1. Header
    let title = Paragraph::new("Git Kanban (Local .git storage)")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
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

        let border_style = if app.active_column == i && !app.input_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(column_titles[i]).border_style(border_style))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

        if app.active_column == i {
            let mut state = ListState::default();
            state.select(Some(app.selected_index));
            f.render_stateful_widget(list, columns_layout[i], &mut state);
        } else {
            f.render_widget(list, columns_layout[i]);
        }
    }

    // 3. Footer / Input
    if app.input_mode {
        let title = if app.is_editing { "Edit Task" } else { "New Task" };

        let input = Paragraph::new(app.input_buffer.as_str())
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(input, chunks[2]);
    } else {
        let help_text = "q:Quit | n:New | e:Edit | d:Delete | Enter:Move | ←/→/↑/↓:Nav";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }
}