mod app;
mod entities;
mod log;
mod render;
mod storage;

use std::{
    io::{self},
    ops::ControlFlow,
};

use app::App;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use self::render::render_app;

fn main() -> Result<()> {
    color_eyre::install()?;

    let storage = storage::AppTreeStorage::load_state()?;

    let app = App::new(storage);

    // let app = load_state()?.map_or_else(App::new, State::into_app);

    // Setup
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;
    enable_raw_mode()?;
    terminal.hide_cursor()?;

    let result = run(app, &mut terminal);

    // Cleanup
    terminal.clear()?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    result
}

fn run(mut app: App, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    loop {
        terminal.draw(|frame| render_app(frame, &mut app))?;

        // let selection = app.selection_index();
        // let contents = app.trees.clone();

        let flow = handle_input(&mut app)?;

        // let selection_changed = selection != app.selection_index();
        // let contents_changed = contents != app.trees;

        app.storage.save()?;

        // if contents_changed || selection_changed {
        //     save_state(&State::from_app(&app))?;
        // }

        if flow == ControlFlow::Break(()) {
            break Ok(());
        }
    }
}

fn handle_input(app: &mut App) -> Result<ControlFlow<()>> {
    // use KeyCode::*;
    
    use ratatui::crossterm::event::KeyCode::*;
    use ratatui::crossterm::event::KeyEventKind;
    use ratatui::crossterm::event;


    if let event::Event::Key(key) = event::read()? {
        match app.state {
            app::AppState::Normal if key.kind == KeyEventKind::Press  => {
                match key.code {
                    Char('q') => return Ok(ControlFlow::Break(())),
                    Char('d') => _ = app.delete_current_task(),
                    Char('g') => app.scroll_to_top(),
                    Char('G') => app.scroll_to_bottom(),
                    Char('n') => {
                        app.init_insert_mode_to_insert_new_task();
                    }
                    Char('e') => _ = app.init_insert_mode_to_edit_task_title(),
                    // Char('u') => app.undo_change(),
                    // Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => app.redo_change(),
                    Enter | Right => app.nest_task(),
                    Esc | Left | Backspace => _ = app.get_back_to_parent(),
                    Up => app.move_selection_up(),
                    Down => app.move_selection_down(),
                    Tab => app.update_done_state(),
                    _ => {}
                }
            }
            app::AppState::InsertTask { parent_id } => {
                match key.code {
                    Esc => app.cancel_insert_mode(),
                    Enter => app.close_insert_mode_inserting_new_task(),
                    _ => {
                        app.text_area.input(key);
                    },
                }
            }
            app::AppState::EditTask { task_id } => {
                match key.code {
                    Esc => app.cancel_insert_mode(),
                    Enter => app.close_insert_mode_updating_task_title(),
                    _ => {
                        app.text_area.input(key);
                    },
                }
            }
            _ => {}
        }
    }

    Ok(ControlFlow::Continue(()))
}
