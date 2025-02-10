use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::{App, AppState};

pub fn render_app(frame: &mut Frame, app: &mut App) {
    let stack_list = {
        let stack = app
            .find_parents_titles()
            .into_iter()
            .rev()
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

    let elements_view_constraint = Constraint::Min(elements_list.len() as u16);

    if app.find_parents_titles().len() > 0 {
        let stack_view_constraint = Constraint::Length(2 + app.find_parents_titles().len() as u16);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([stack_view_constraint, elements_view_constraint])
            .split(entire_area);

        frame.render_stateful_widget(stack_list, layout[0], &mut app.stack_list);
        frame.render_stateful_widget(elements_list, layout[1], &mut app.elements_list);
    } else {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([elements_view_constraint])
            .split(entire_area);

        frame.render_stateful_widget(elements_list, layout[0], &mut app.elements_list);
    }

    if let AppState::INSERT{ .. } = app.state {
        let popup_block = Block::default()
            .title("Enter a new key-value pair")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, frame.area());

        app.text_area.set_block(popup_block);

        frame.render_widget(&app.text_area, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
