use ratatui::{
  prelude::*, 
  widgets::*,
  widgets::block::title::*,
  symbols::border,
};

use crate::app::App;

const INFO_TEXT: &str =
  "(Esc) quit | (↑) move up | (↓) move down | (R) Refresh | (Space) Aknowledge | (Enter) Open incident in browser";

const SPLASH_TEXT: &str = " ____   __    ___  ____  ____    ____  _  _  ____  _  _ \n(  _ \\ / _\\  / __)(  __)(  _ \\  (    \\/ )( \\(_  _)( \\/ )\n ) __//    \\( (_ \\ ) _)  )   /   ) D () \\/ (  )(   )  / \n(__)  \\_/\\_/ \\___/(____)(__\\_)  (____/\\____/ (__) (__/  ";

pub fn ui(f: &mut Frame, app: &mut App) {
  let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());

  app.set_colors();

  render_table(f, app, rects[0]);

  render_scrollbar(f, app, rects[0]);

  render_footer(f, app, rects[1]);
}

pub fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
  let header_style = Style::default()
    .fg(app.colors.header_fg)
    .bg(app.colors.header_bg);
  let selected_style = Style::default()
    .add_modifier(Modifier::REVERSED)
    .fg(app.colors.selected_style_fg);

  let header = ["STATUS", "SERVICE - SUMMARY", "CREATED AT"]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(header_style)
    .height(1);
  let rows = app.items.iter().enumerate().map(|(i, data)| {
    let color = match i % 2 {
      0 => app.colors.normal_row_color,
      _ => app.colors.alt_row_color,
    };
    let item = data.ref_array();
    item.into_iter()
      .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
      .collect::<Row>()
      .style(Style::new().fg(app.colors.row_fg).bg(color))
      .height(4)
  });

  let bar = " █ ";
  let title_str:String = format!(" - Pager Duty TUI - ");
  let title = Title::from(title_str.bold());
  let t = Table::new(
    rows,
    [
      // + 1 is for padding.
      Constraint::Length(app.longest_item_lens.0 + 1),
      Constraint::Min(app.longest_item_lens.1 + 1),
      Constraint::Min(app.longest_item_lens.2),
    ],
  )
  .header(header)
  .highlight_style(selected_style)
  .block(
    Block::default()
      .title(title.alignment(Alignment::Center))
      .borders(Borders::ALL)
      .border_style(Style::new().fg(app.colors.footer_border_color))
      .border_type(BorderType::Double),
  )
  .highlight_symbol(Text::from(vec![
    "".into(),
    bar.into(),
    bar.into(),
    "".into(),
  ]))
  .bg(app.colors.buffer_bg)
  .highlight_spacing(HighlightSpacing::Always);
  f.render_stateful_widget(t, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
  f.render_stateful_widget(
    Scrollbar::default()
      .orientation(ScrollbarOrientation::VerticalRight)
      .begin_symbol(None)
      .end_symbol(None),
    area.inner(&Margin {
      vertical: 1,
      horizontal: 1,
    }),
    &mut app.scroll_state,
  );
}

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
  let mut footer_text: String = String::from("");
  if app.refreshing {
    footer_text = String::from(" <- REFRESHING -> ");
  } else {
    footer_text = String::from(INFO_TEXT);
  }

  let info_footer = Paragraph::new(Line::from(footer_text))
    .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
    .centered()
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(app.colors.footer_border_color))
        .border_type(BorderType::Double),
    );
  f.render_widget(info_footer, area);
}

pub fn splash_screen(f: &mut Frame) {
  let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());

  let block = Block::default()
    .borders(Borders::ALL)
    .border_set(border::THICK);

  let splash_text = Paragraph::new(SPLASH_TEXT)
    .green()
    .on_dark_gray()
    .alignment(Alignment::Center)
    .block(block);
  
  f.render_widget(splash_text, rects[0]);
}
