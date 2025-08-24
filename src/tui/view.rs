use super::common::*;
use super::model::{EntryItem, MessageKind, Model, Session};
use ratatui::style::Color;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListDirection, ListItem, Padding, Paragraph, Wrap},
};

use crate::domain::EntryKind;

const PANE_TITLE_FG_COLOR: Color = Color::Black;
const PRIMARY_COLOR: Color = Color::LightBlue;
const INFO_MESSAGE_COLOR: Color = Color::LightBlue;
const ERROR_MESSAGE_COLOR: Color = Color::LightRed;
const HELP_COLOR: Color = Color::Yellow;

const TITLE: &str = " atls ";

pub fn view(model: &mut Model, frame: &mut Frame) {
    if model.terminal_too_small {
        render_terminal_too_small_view(&model.terminal_dimensions, frame);
        return;
    }

    match model.active_pane {
        Pane::Explorer => render_explorer_view(model, frame),
        Pane::Help => render_help_pane(model, frame),
    }
}

fn render_terminal_too_small_view(dimensions: &TerminalDimensions, frame: &mut Frame) {
    let message = format!(
        r#"
Terminal size too small:
  Width = {} Height = {}

Minimum dimensions needed:
  Width = {} Height = {}

Press (q/<ctrl+c>/<esc> to exit)
"#,
        dimensions.width, dimensions.height, MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT
    );

    let p = Paragraph::new(message)
        .block(Block::bordered())
        .style(Style::new().fg(PRIMARY_COLOR))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center);

    frame.render_widget(p, frame.area());
}

fn render_explorer_view(model: &mut Model, frame: &mut Frame) {
    let main_rect = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![Constraint::Min(10), Constraint::Length(1)])
        .split(frame.area());

    render_explorer_pane(model, frame, main_rect[0]);
    render_status_line(model, frame, main_rect[1]);
}

fn render_explorer_pane(model: &mut Model, frame: &mut Frame, rect: Rect) {
    let rect = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![Constraint::Length(1), Constraint::Min(9)])
        .split(rect);

    let mut header_spans = vec![];
    for (i, session) in model.sessions.iter().enumerate() {
        let mut span_style = Style::new();
        if session.is_initialized() && i != model.current_session_index {
            span_style = span_style.underlined();
        }

        if i == model.current_session_index {
            span_style = span_style.bold().fg(Color::Black).bg(Color::Blue);
        }

        let span = Span::styled(format!("{}", i + 1), span_style);

        header_spans.push(span);

        if i < 3 {
            header_spans.push(Span::from(" "));
        }
    }

    header_spans.push(Span::from(" "));

    // TODO: can be made better
    // gets a mutable reference to the entire current session
    match model.current_session_mut() {
        Session::Uninitialized => {
            let header = Line::from(header_spans);
            frame.render_widget(header, rect[0]);
        }
        Session::Initialized {
            path,
            entries,
            state,
        } => {
            header_spans.push(Span::styled(
                path.to_string_lossy(),
                Style::new().fg(Color::Blue),
            ));

            let header = Line::from(header_spans);

            let selected_index = state.selected();
            let items: Vec<ListItem> = entries
                .iter()
                .enumerate()
                .map(|(i, entry)| entry_to_list_item(entry, selected_index == Some(i)))
                .collect();

            let list = List::new(items)
                .block(Block::new().padding(Padding::new(0, 0, 1, 0)))
                .direction(ListDirection::TopToBottom);
            frame.render_widget(header, rect[0]);
            frame.render_stateful_widget(list, rect[1], state);
        }
    }
}

fn render_status_line(model: &Model, frame: &mut Frame, rect: Rect) {
    let mut status_bar_lines = vec![Span::styled(
        TITLE,
        Style::new()
            .bold()
            .bg(PRIMARY_COLOR)
            .fg(PANE_TITLE_FG_COLOR),
    )];

    if let Some(msg) = &model.user_msg {
        let span = match msg.kind {
            MessageKind::Info => Span::styled(
                format!(" {}", msg.value),
                Style::new().fg(INFO_MESSAGE_COLOR),
            ),
            MessageKind::Error => Span::styled(
                format!(" {}", msg.value),
                Style::new().fg(ERROR_MESSAGE_COLOR),
            ),
        };

        status_bar_lines.push(span);
    }

    if model.debug {
        status_bar_lines.push(Span::from(format!(
            " [session: {}]",
            model.current_session_index
        )));
        match model.current_session() {
            Session::Uninitialized => {}
            Session::Initialized {
                path: _,
                entries: _,
                state,
            } => {
                status_bar_lines.push(Span::from(format!(" [selected: {:?}]", state.selected())));
            }
        }
        status_bar_lines.push(Span::from(format!(" [render: {}]", model.render_counter)));
        status_bar_lines.push(Span::from(format!(" [event: {}]", model.event_counter)));
        status_bar_lines.push(Span::from(format!(
            " [dimensions: {}x{}] ",
            model.terminal_dimensions.width, model.terminal_dimensions.height
        )));
    }

    let status_bar_text = Line::from(status_bar_lines);

    let status_bar = Paragraph::new(status_bar_text).block(Block::default());

    frame.render_widget(&status_bar, rect);
}

fn render_help_pane(model: &Model, frame: &mut Frame) {
    let main_rect = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![Constraint::Min(10), Constraint::Length(1)])
        .split(frame.area());

    let help_content = get_help_content();
    let lines: Vec<Line> = help_content
        .lines()
        .skip(model.help_scroll)
        .map(Line::raw)
        .collect();

    let title = " help ";

    let help_widget = Paragraph::new(lines)
        .block(
            Block::new()
                .title_style(Style::new().bold().bg(HELP_COLOR).fg(PANE_TITLE_FG_COLOR))
                .title(title)
                .padding(Padding::new(1, 0, 1, 0)),
        )
        .style(Style::new().white().on_black())
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);

    frame.render_widget(&help_widget, main_rect[0]);
    render_status_line(model, frame, main_rect[1]);
}

fn entry_to_list_item(item: &EntryItem, is_selected: bool) -> ListItem<'_> {
    let color = match item.entry.kind() {
        EntryKind::File => Color::White,
        EntryKind::Directory => Color::LightRed,
        EntryKind::Symlink => Color::Magenta,
        EntryKind::Unknown => Color::Gray,
    };

    let base_style = Style::new().fg(color);
    let highlight_style = if is_selected {
        Style::new().bg(Color::Blue).fg(Color::Black).bold()
    } else {
        base_style
    };

    let spans = if item.marked {
        vec![
            Span::styled("+", Style::new().bg(Color::Yellow).fg(Color::Black)),
            Span::from(item.entry.path_str()).style(highlight_style),
        ]
    } else {
        vec![
            " ".into(),
            Span::from(item.entry.path_str()).style(highlight_style),
        ]
    };
    let line = Line::from(spans);

    ListItem::new(line)
}
