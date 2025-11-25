mod app;
mod io;
mod ui;

use crate::app::App;
use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::*};

fn main() -> Result<()> {
    // 1. Setup paths
    let git_dir =
        io::find_git_dir().context("Could not find .git directory. Run this inside a git repo.")?;
    let data_path = git_dir.join("git-kanban.json");

    // 2. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Initialize App
    let mut app = App::new(data_path);

    // 4. Run Loop
    let res = run_app(&mut terminal, &mut app);

    // 5. Restore Terminal
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
            // Priority 1: Input Mode
            if app.input_mode {
                match key.code {
                    KeyCode::Enter => app.submit_input(),
                    KeyCode::Esc => app.cancel_input(),
                    KeyCode::Char(c) => app.input_buffer.push(c),
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    _ => {}
                }
            }
            // Priority 2: View Mode (Popup active)
            else if app.view_mode {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('v') | KeyCode::Char('q') | KeyCode::Enter => {
                        app.toggle_view_mode()
                    }
                    _ => {}
                }
            }
            // Priority 3: Normal Navigation
            else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('n') => app.start_adding(),
                    KeyCode::Char('e') => app.start_editing(),
                    KeyCode::Char('v') => app.toggle_view_mode(),
                    KeyCode::Char('d') => app.delete_current_task(),

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
