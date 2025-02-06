mod app;
mod entities;
mod history;
mod log;
mod render;
mod repository;

use std::{
    io::{self},
    ops::ControlFlow,
};

use app::App;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use self::render::render_app;

fn main() -> Result<()> {
    color_eyre::install()?;

    let repository = repository::AppTreeRepository::load_state()?;

    let app = App::new(repository);

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

        app.repository.save()?;

        // if contents_changed || selection_changed {
        //     save_state(&State::from_app(&app))?;
        // }

        if flow == ControlFlow::Break(()) {
            break Ok(());
        }
    }
}

fn handle_input(app: &mut App) -> Result<ControlFlow<()>> {
    use KeyCode::*;

    if let app::AppState::INSERT(_) = app.state {
        if let ratatui::crossterm::event::Event::Key(key) = ratatui::crossterm::event::read()? {
            if key.code == ratatui::crossterm::event::KeyCode::Esc {
                app.cancel_insert_mode();
            } else if key.code == ratatui::crossterm::event::KeyCode::Enter {
                app.close_insert_mode_updating_task_title();
            } else {
                app.text_area.input(key);
            }
        }
        return Ok(ControlFlow::Continue(()));
    }

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                Char('q') => return Ok(ControlFlow::Break(())),
                Char('d') => app.delete_current_task(),
                Char('n') => app.add_new_task(),
                Char('g') => app.scroll_to_top(),
                Char('G') => app.scroll_to_bottom(),
                Char('e') => app.init_insert_mode_to_edit_a_task_title(),
                // Char('u') => app.undo_change(),
                // Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => app.redo_change(),
                Enter | Right => app.nest_task(),
                Esc | Left | Backspace => app.get_back_to_parent(),
                Up => app.move_selection_up(),
                Down => app.move_selection_down(),
                _ => {}
            }
        }
    }

    Ok(ControlFlow::Continue(()))
}
