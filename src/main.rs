mod app;
mod io;
mod ui;

use crate::app::{App, EditFocus};
use anyhow::Result;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::*};

fn main() -> Result<()> {
    let data_path = io::find_storage_path()?;
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(data_path);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            // 1. Edit Mode
            if app.edit_mode {
                match key.code {
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.save_edit_changes()
                    }
                    KeyCode::Esc => app.close_edit_mode(),
                    KeyCode::Tab => app.toggle_edit_focus(),

                    // Input based on focus
                    _ => match app.edit_focus {
                        EditFocus::Title => match key.code {
                            KeyCode::Char(c) => app.enter_char_edit_title(c),
                            KeyCode::Backspace => app.delete_char_edit_title(),
                            KeyCode::Enter => app.toggle_edit_focus(), // Enter moves to desc
                            _ => {}
                        },
                        EditFocus::Description => {
                            app.description_editor.input(key);
                        }
                    },
                }
            }
            // 2. View Mode (Read Only)
            else if app.view_mode {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('v') | KeyCode::Char('q') | KeyCode::Enter => {
                        app.close_view_mode()
                    }
                    _ => {}
                }
            }
            // 3. Quick Add (Footer)
            else if app.input_mode {
                match key.code {
                    KeyCode::Enter => app.submit_input(),
                    KeyCode::Esc => app.cancel_input(),
                    KeyCode::Char(c) => app.enter_char_footer(c),
                    KeyCode::Backspace => app.delete_char_footer(),
                    KeyCode::Left => app.move_cursor_left_footer(),
                    KeyCode::Right => app.move_cursor_right_footer(),
                    _ => {}
                }
            }
            // 4. Delete Confirm
            else if app.delete_mode {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => app.confirm_delete(),
                    KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => app.cancel_delete(),
                    _ => {}
                }
            }
            // 5. Navigation
            else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => app.start_adding(),
                    KeyCode::Char('e') => app.open_edit_mode(),
                    KeyCode::Char('v') => app.open_view_mode(),
                    KeyCode::Char('d') => app.prompt_delete(),

                    KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.move_task_up()
                    }
                    KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.move_task_down()
                    }

                    KeyCode::Left => app.prev_column(),
                    KeyCode::Right => app.next_column(),
                    KeyCode::Up => app.prev_item(),
                    KeyCode::Down => app.next_item(),
                    KeyCode::Enter => app.move_current_task(),
                    _ => {}
                }
            }
        }
    }
}
