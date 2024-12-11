use ratatui::{
  prelude::*, symbols::border, widgets::{block::title::*, *}
};

use crate::{app::App, pagerduty::Incident};

const INFO_TEXT: &str =
  "(Esc) Quit | (↑/↓/🏠) Navigate | (R) Refresh | (Space) Ack | (A) Ack service | (G) Show all | (Enter) Open";

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

  // Creating rows for table
  let mut rows: Vec<Row> = Vec::new();
  for i in 0..app.items.len() {
    let item = app.items.get(i).unwrap();
    // If item is ack and the ack are not hide
    if (!!!*item.triggered() && !!!app.hide_ack ) || (*item.triggered()) {
      let color:Color;

      if *item.triggered(){
        color = match i % 2 {
          0 => app.colors.triggered_normal_color,
          _ => app.colors.triggered_alt_color,
        };
      } else {
        color = match i % 2 {
          0 => app.colors.normal_row_color,
          _ => app.colors.alt_row_color,
        };
      }
      
      rows.push(item.ref_array().into_iter()
        .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
        .collect::<Row>()
        .style(Style::new().fg(app.colors.row_fg).bg(color))
        .height(4));
    }
  }
  if rows.len() == 0 {
    let empty_item: Incident = Incident {
      id: String::from("---------"),
      summary: String::from(" - NO INCIDENTS | TIME FOR A BREAK - "),
      status: String::from("---------"),
      service: String::from(""),
      created_at: String::from("---------"),
      triggered: false, 
    };
    rows.push(empty_item.ref_array().into_iter()
      .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
      .collect::<Row>()
      .style(Style::new().fg(app.colors.row_fg).bg(app.colors.triggered_normal_color))
      .height(4));
  }

  let bar = " █ ";
  let title_str:String = format!(" - Pager Duty TUI - ");
  let title = Title::from(title_str.bold());
  let t = Table::new(
    rows.into_iter(),
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
      .border_type(BorderType::Rounded),
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
  let footer_text: String;
  let color_bg: Color;
  let border: Borders;
  let padding: Padding;
  if app.refreshing {
    border = Borders::NONE;
    footer_text = String::from(" <- REFRESHING -> ");
    color_bg = Color::Yellow;
    padding = Padding::new(0, 0, 1, 0);
  } else {
    border = Borders::ALL;
    footer_text = String::from(INFO_TEXT);
    color_bg = app.colors.buffer_bg;
    padding = Padding::new(0, 0, 0, 0);
  }

  let info_footer = Paragraph::new(Line::from(footer_text))
    .style(Style::new().fg(app.colors.row_fg).bg(color_bg))
    .centered()
    .block(
      Block::default()
        .borders(border)
        .border_style(Style::new().fg(app.colors.footer_border_color))
        .border_type(BorderType::Rounded)
        .padding(padding)
        .bold(),
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
