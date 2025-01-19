mod disk;
mod render;

use std::{
    io::{self},
    ops::ControlFlow,
};

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use disk::save_state;
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

use self::{
    disk::{State, load_state},
    render::render_app,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let app = load_state()?.map_or_else(App::new, State::into_app);

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

        let flow = handle_input(&mut app)?;

        if app.changed {
            save_state(&State::from_app(&app))?;
            app.changed = false;
        }

        if flow == ControlFlow::Break(()) {
            break Ok(());
        }
    }
}

fn handle_input(app: &mut App) -> Result<ControlFlow<()>> {
    use KeyCode::*;

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                Char('q') => return Ok(ControlFlow::Break(())),
                Char('d') => app.delete_selected_line(),
                Char('n') => app.add_line_below(),
                Up => app.move_selection_up(),
                Down => app.move_selection_down(),
                _ => {}
            }
        }
    }

    Ok(ControlFlow::Continue(()))
}

struct App {
    changed: bool,
    names: Vec<String>,
    list: ListState,
}

impl App {
    fn new() -> Self {
        let names = vec![
            "Alice".into(),
            "Bob".into(),
            "Carol".into(),
            "Dave".into(),
            "Eve".into(),
        ];
        Self::from_names(names)
    }

    fn from_names(names: Vec<String>) -> Self {
        let mut list = ListState::default();
        list.select(Some(0));
        Self {
            names,
            list,
            changed: false,
        }
    }

    fn current(&self) -> usize {
        self.list.selected().unwrap()
    }

    fn set_current(&mut self, index: usize) {
        self.changed |= self.current() != index;
        self.list.select(Some(index));
    }

    fn move_selection_up(&mut self) {
        let next = if self.current() == 0 {
            self.names.len() - 1 // Wrap to end
        } else {
            self.current() - 1
        };
        self.set_current(next);
    }

    fn move_selection_down(&mut self) {
        let next = if self.current() + 1 >= self.names.len() {
            0 // Wrap to start
        } else {
            self.current() + 1
        };
        self.set_current(next);
    }

    fn add_line_below(&mut self) {
        self.names.insert(self.current() + 1, String::from("oiee"));
        self.move_selection_down();
        self.changed = true;
    }

    fn delete_selected_line(&mut self) {
        self.names.remove(self.current());
        self.move_selection_up();
        self.changed = true;
    }
}
