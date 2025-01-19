use ratatui::{
    Frame,
    layout::Margin,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::App;

pub fn render_app(frame: &mut Frame, app: &mut App) {
    let area = frame.area().inner(Margin::new(3, 1));

    // Create list items
    let items: Vec<ListItem> = app
        .names
        .iter()
        .map(|name| ListItem::new(Line::from(name.clone())))
        .collect();

    // Create a List widget
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Names ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">> ");

    // Render the list with its state
    frame.render_stateful_widget(list, area, &mut app.list);
}
