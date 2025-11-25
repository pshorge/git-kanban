use crate::app::App;
use ratatui::{prelude::*, widgets::*};

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

        let border_style = if app.active_column == i && !app.input_mode {
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

    // 3. Footer / Input
    if app.input_mode {
        let title = if app.is_editing {
            "Edit Task"
        } else {
            "New Task"
        };

        let input = Paragraph::new(app.input_buffer.as_str())
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(input, chunks[2]);
    } else {
        let help_text = "q:Quit | n:New | e:Edit | v:View | d:Delete | Shift+↑/↓:Move | Enter:Next | ←/→/↑/↓:Nav";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    // 4. Details Popup (Draw on top of everything if view_mode is active)
    if app.view_mode {
        let block = Block::default()
            .title("Task Details")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        let text = app.get_current_task_title();
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

        let area = centered_rect(60, 40, f.area()); // 60% width, 40% height

        // Clear the area underneath the popup so background doesn't shine through
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    // 5. Delete Confirmation Popup (Draw on top if delete_mode is active)
    if app.delete_mode {
        let block = Block::default()
            .title("Confirmation")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red).bg(Color::Black)); // Red border for warning

        let text = vec![
            Line::from("Are you sure you want to delete this task?"),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Y",
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                ),
                Span::raw(" to confirm / "),
                Span::styled("N", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        let area = centered_rect(40, 20, f.area()); // Smaller popup

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

/// Helper function to center a rect in the terminal
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
