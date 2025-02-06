use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::App;

pub fn render_app(frame: &mut Frame, app: &mut App) {
    let stack_list = {
        let stack = app
            .find_parents_titles()
            .into_iter()
            .map(String::from)
            .map(Line::from)
            .map(ListItem::new);

        List::new(stack)
            .block(
                Block::default()
                    .title(" Stack ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(" -- ")
    };

    let elements_list = {
        let viewed_nodes = app.find_tasks_to_display();

        let elements = viewed_nodes
            .iter()
            .map(|item| item.title.clone())
            .map(Line::from)
            .map(ListItem::new);

        List::new(elements)
            .block(
                Block::default()
                    .title(" Elements ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(" > ")
    };

    let entire_area = frame.area().inner(Margin::new(3, 1));

    let stack_view_constraint = Constraint::Length(5);
    let elements_view_constraint = Constraint::Min(elements_list.len() as u16);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([stack_view_constraint, elements_view_constraint])
        .split(entire_area);

    frame.render_stateful_widget(stack_list, layout[0], &mut app.stack_list);
    frame.render_stateful_widget(elements_list, layout[1], &mut app.elements_list);
}
