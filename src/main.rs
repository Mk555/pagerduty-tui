#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

use std::{error::Error, io};

use ratatui::prelude::*;

use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod app;
use app::{App,run_app};
mod actions;
mod config;
use config::AppConfig;
mod pagerduty;
use pagerduty::PagerDuty;
mod utils;
mod ui;
use ui::splash_screen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let app_config:AppConfig = AppConfig::new();

  // Init PD
  let pd: PagerDuty = PagerDuty::new(&app_config.get_pagerduty_api_key());

  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let _ = terminal.draw(|f,|splash_screen(f));

  // create app and run it
  let app = App::new(pd).await;
  let res = run_app(&mut terminal, app).await;

  // restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("{err:?}");
  }

  Ok(())
}
