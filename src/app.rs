use ratatui::{
  prelude::*, 
  widgets::*,
};
use std::io;
use tokio::sync::mpsc::{self, UnboundedSender,UnboundedReceiver};
use unicode_width::UnicodeWidthStr;
use style::palette::tailwind;

use crate::config::AppConfig;
use crate::pagerduty::{Incident, PagerDuty};
use crate::actions::{Action,handle_event,update,REFRESH_RATE};
use crate::ui::{ui,splash_screen};

const ITEM_HEIGHT: usize = 4;

const PALETTES: [tailwind::Palette; 1] = [
  tailwind::GREEN,
];

pub struct TableColors {
  pub buffer_bg: Color,
  pub header_bg: Color,
  pub header_fg: Color,
  pub row_fg: Color,
  pub selected_style_fg: Color,
  pub normal_row_color: Color,
  pub alt_row_color: Color,
  pub footer_border_color: Color,
  pub triggered_normal_color: Color,
  pub triggered_alt_color: Color,
}

impl TableColors {
  const fn new(color: &tailwind::Palette) -> Self {
    Self {
      buffer_bg: tailwind::SLATE.c950,
      header_bg: color.c900,
      header_fg: tailwind::SLATE.c200,
      row_fg: tailwind::SLATE.c200,
      selected_style_fg: color.c400,
      normal_row_color: tailwind::SLATE.c950,
      alt_row_color: tailwind::SLATE.c900,
      footer_border_color: color.c400,
      triggered_normal_color: tailwind::RED.c700,
      triggered_alt_color: tailwind::RED.c500,
    }
  }
}

pub struct App {
  pub state: TableState,
  pub items: Vec<Incident>,
  pub pager_duty: PagerDuty,
  pub longest_item_lens: (u16, u16, u16), // order is (status,summary,created_at)
  pub scroll_state: ScrollbarState,
  pub colors: TableColors,
  pub color_index: usize,
  pub action_tx: UnboundedSender<Action>,
  pub action_rx: UnboundedReceiver<Action>,
  pub refreshing: bool,
  pub should_quit: bool,
  pub refresh_rate: Option<i64>,
  pub ticker: i64,
}

impl App {
  pub async fn new(pd: PagerDuty, config: &AppConfig) -> Self {

    let data_vec = pd.get_incidents().await.expect("Error getting incidents from PagerDuty");
    let (action_tx, action_rx) = mpsc::unbounded_channel();

    Self {
      state: TableState::default().with_selected(0),
      longest_item_lens: constraint_len_calculator(&data_vec),
      scroll_state: ScrollbarState::new((data_vec.len() - 1) * ITEM_HEIGHT),
      colors: TableColors::new(&PALETTES[0]),
      color_index: 0,
      items: data_vec,
      pager_duty: pd,
      should_quit: false,
      refreshing: false,
      action_tx,
      action_rx,
      refresh_rate: *config.get_refresh_rate(),
      ticker: 0,
    }
  }
  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i >= self.items.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
    self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
  }

  pub fn previous(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          self.items.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
    self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
  }

  pub fn set_colors(&mut self) {
    self.colors = TableColors::new(&PALETTES[self.color_index]);
  }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {

  let _ = terminal.draw(|f,|splash_screen(f));

  let _task = handle_event(&app, app.action_tx.clone());

  loop {
    terminal.draw(|f| ui(f, &mut app))?;

    if let Some(action) = app.action_rx.recv().await {
      update(&mut app, action).await;
    }
    if app.should_quit {
      break;
    }

    // REFRESH EVERY X SECOND
    if app.ticker >= app.refresh_rate.unwrap_or(60) * ( 1000 / REFRESH_RATE) {
      app.refreshing = true;
      terminal.draw(|f| ui(f, &mut app))?;

      app.items = app.pager_duty.get_incidents().await.unwrap();
      
      app.ticker = 0;

      app.refreshing = false;
      terminal.draw(|f| ui(f, &mut app))?;
    } else {
      app.ticker += 1;
    }
  }
  Ok(())
}

pub fn constraint_len_calculator(items: &[Incident]) -> (u16, u16, u16) {
  let name_len = items
    .iter()
    .map(Incident::status)
    .map(UnicodeWidthStr::width)
    .max()
    .unwrap_or(0);
  let address_len = items
    .iter()
    .map(Incident::summary)
    .flat_map(str::lines)
    .map(UnicodeWidthStr::width)
    .max()
    .unwrap_or(0);
  let email_len = items
    .iter()
    .map(Incident::created_at)
    .map(UnicodeWidthStr::width)
    .max()
    .unwrap_or(0);

  #[allow(clippy::cast_possible_truncation)]
  (name_len as u16, address_len as u16, email_len as u16)
}
