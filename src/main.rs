mod app;
mod io;
mod ui;

use crate::app::App;
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
            // Priority 1: Unified Description Editor
            if app.edit_desc_mode {
                match key.code {
                    // Esc saves and closes (easy read/edit flow)
                    KeyCode::Esc => app.close_description(),
                    _ => {
                        // Pass key to the tui-textarea widget
                        app.description_editor.input(key);
                    }
                }
            }
            // Priority 2: Title Input Mode
            else if app.input_mode {
                match key.code {
                    KeyCode::Enter => app.submit_input(),
                    KeyCode::Esc => app.cancel_input(),
                    KeyCode::Char(c) => app.enter_char(c),
                    KeyCode::Backspace => app.delete_char(),
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    _ => {}
                }
            }
            // Priority 3: Delete Confirmation
            else if app.delete_mode {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => app.confirm_delete(),
                    KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => app.cancel_delete(),
                    _ => {}
                }
            }
            // Priority 4: Normal Navigation
            else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => app.start_adding(),
                    KeyCode::Char('e') => app.start_editing(),

                    // 'v' now opens the unified editor/viewer
                    KeyCode::Char('v') => app.open_description(),

                    KeyCode::Char('d') => app.prompt_delete(),

                    // Reordering with Shift
                    KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.move_task_up()
                    }
                    KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.move_task_down()
                    }

                    // Standard Navigation
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
