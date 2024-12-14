use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn split_rect(percent: u16, r: Rect, direction: Direction) -> (Rect, Rect) {
    let rects = Layout::default()
        .direction(direction)
        .constraints([Constraint::Percentage(percent), Constraint::Fill(1)])
        .split(r);
    (rects[0], rects[1])
}

// ANCHOR: centered_rect
/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
// ANCHOR_END: centered_rect

pub fn sub_rect(
    percent_x: u16,
    percent_width: u16,
    percent_y: u16,
    percent_high: u16,
    r: Rect,
) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(percent_x),
            Constraint::Percentage(percent_width),
            Constraint::Percentage(100 - percent_x - percent_width),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(percent_y),
            Constraint::Percentage(percent_high),
            Constraint::Percentage(100 - percent_y - percent_high),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
