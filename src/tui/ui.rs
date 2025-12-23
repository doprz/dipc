use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::{
    app::{App, PaletteEntry, Panel, HELP_POPUP, HELP_TEXT, PALETTES},
    utils::truncate,
};

const CLR_ACCENT: Color = Color::Green; // Focused borders, titles
const CLR_SUCCESS: Color = Color::Green; // Selected items, success
const CLR_TEXT: Color = Color::White; // Light text
const CLR_TEXT_DIM: Color = Color::Gray; // Dimmed text
const CLR_SURFACE: Color = Color::Rgb(50, 50, 50); // Highlight background
const CLR_BORDER: Color = Color::DarkGray; // Unfocused borders
const CLR_WARNING: Color = Color::Yellow; // Yellow, warning

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(frame.area());

    let main_area = chunks[0];
    let help_bar = chunks[1];

    // Main layout: left and right columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    // Left column: Files (top) and Palette (bottom)
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(cols[0]);

    // Right column: Preview (top) and Output (bottom)
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(cols[1]);

    render_files(frame, app, left[0]);
    render_palette(frame, app, left[1]);
    render_preview(frame, app, right[0]);
    render_output(frame, app, right[1]);

    // Help bar
    let help = Paragraph::new(HELP_TEXT)
        .style(Style::default().fg(CLR_TEXT_DIM))
        .alignment(Alignment::Center);
    frame.render_widget(help, help_bar);

    // Overlays
    if app.show_help {
        render_help_popup(frame);
    }
    if app.input_mode {
        render_input_popup(frame, app);
    }
}

fn render_files(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.panel == Panel::Files;
    let title = format!(" Files ({}) ", app.current_dir.display());

    let items: Vec<ListItem> = app
        .file_entries
        .iter()
        .map(|e| {
            let selected = app.selected_files.contains(&e.path);
            let (icon, color) = if e.name == ".." || e.is_dir {
                ("", CLR_ACCENT)
            } else if selected {
                ("", CLR_SUCCESS)
            } else {
                ("", CLR_TEXT)
            };
            let marker = if selected { "●" } else { " " };
            let style = Style::default().fg(color);
            ListItem::new(format!(" {} {} {}", marker, icon, e.name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block(&title, focused))
        .highlight_style(Style::default().bg(CLR_SURFACE).bold())
        .highlight_symbol("▌");

    frame.render_stateful_widget(list, area, &mut app.file_state);
}

fn render_palette(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.panel == Panel::Palette;
    let title = " Palette ".to_string();

    let items: Vec<ListItem> = app
        .palette_entries
        .iter()
        .map(|entry| match entry {
            PaletteEntry::Palette { name, idx } => {
                let selected = *idx == app.selected_palette_idx;
                let marker = if selected { "▼" } else { "▶" };
                let style = if selected {
                    Style::default().fg(CLR_ACCENT).bold()
                } else {
                    Style::default().fg(CLR_TEXT)
                };
                ListItem::new(format!(" {marker} {name}")).style(style)
            }
            PaletteEntry::Variation { name, .. } => {
                let selected = app.selected_variations.contains(name);
                let marker = if selected { "●" } else { "○" };
                let color = if selected { CLR_SUCCESS } else { CLR_TEXT_DIM };
                ListItem::new(format!("    {marker} {name}")).style(Style::default().fg(color))
            }
        })
        .collect();

    let list = List::new(items)
        .block(block(&title, focused))
        .highlight_style(Style::default().bg(CLR_SURFACE))
        .highlight_symbol("▌");

    frame.render_stateful_widget(list, area, &mut app.palette_state);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let palette_name = &PALETTES[app.selected_palette_idx].0;
    let var_info = if app.selected_variations.is_empty() {
        String::from("all variations")
    } else {
        let count = app.selected_variations.len();
        format!("{} variation{}", count, if count == 1 { "" } else { "s" })
    };
    let title = format!(
        " {} · {} · {} colors ",
        palette_name,
        var_info,
        app.preview_colors.len()
    );

    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let cols = (inner.width / 18).max(1) as usize;

    let mut lines: Vec<Line> = Vec::new();
    for chunk in app.preview_colors.chunks(cols) {
        let mut spans = Vec::new();
        for (name, [r, g, b]) in chunk {
            spans.push(Span::styled(
                "  ",
                Style::default().bg(Color::Rgb(*r, *g, *b)),
            ));
            spans.push(Span::styled(
                format!(" {:<12} ", truncate(name, 12)),
                Style::default().fg(CLR_TEXT),
            ));
        }
        lines.push(Line::from(spans));
        lines.push(Line::from(""));
    }

    let para = Paragraph::new(lines).block(block(&title, false));
    frame.render_widget(para, area);
}

fn render_output(frame: &mut Frame, app: &App, area: Rect) {
    let file_count = app.selected_files.len();
    let title = format!(
        " Output · {} file{} selected ",
        file_count,
        if file_count == 1 { "" } else { "s" }
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);

    // Block
    frame.render_widget(block(&title, false), area);

    // Output filenames
    let previews = app.output_preview();
    let mut lines: Vec<Line> = previews
        .iter()
        .map(|s| {
            Line::from(Span::styled(
                format!("  {s}"),
                Style::default().fg(CLR_TEXT_DIM),
            ))
        })
        .collect();

    if app.selected_files.len() > 5 {
        lines.push(Line::from(Span::styled(
            format!("  ... and {} more", app.selected_files.len() - 5),
            Style::default().fg(CLR_TEXT_DIM).italic(),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, chunks[0]);

    // Status line
    let status = if app.selected_files.is_empty() {
        Span::styled("Select files to process", Style::default().fg(CLR_WARNING))
    } else {
        Span::styled("Press Enter to process", Style::default().fg(CLR_SUCCESS))
    };
    let status_line = Paragraph::new(status).alignment(Alignment::Center);
    frame.render_widget(status_line, chunks[1]);
}

fn render_help_popup(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());
    frame.render_widget(Clear, area);

    let popup = Paragraph::new(HELP_POPUP)
        .style(Style::default().fg(CLR_TEXT))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CLR_ACCENT))
                .title(" Help ")
                .title_style(Style::default().fg(CLR_ACCENT).bold()),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(popup, area);
}

fn render_input_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 15, frame.area());
    frame.render_widget(Clear, area);

    let input = Paragraph::new(app.input_buf.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CLR_ACCENT))
                .title(" Enter path ")
                .title_style(Style::default().fg(CLR_ACCENT).bold()),
        )
        .style(Style::default().fg(CLR_TEXT));
    frame.render_widget(input, area);

    frame.set_cursor_position(Position::new(
        area.x + app.input_buf.len() as u16 + 1,
        area.y + 1,
    ));
}

fn block(title: &str, focused: bool) -> Block<'_> {
    let border_color = if focused { CLR_ACCENT } else { CLR_BORDER };
    let title_style = if focused {
        Style::default().fg(CLR_ACCENT).bold()
    } else {
        Style::default().fg(CLR_TEXT)
    };

    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(title_style)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
