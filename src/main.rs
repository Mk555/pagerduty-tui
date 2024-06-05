#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

use std::{env, error::Error, io};

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
mod selfupdate;
use selfupdate::update_bin;
mod utils;
mod ui;
use tokio::task::spawn_blocking;
use ui::splash_screen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();
  if args.len() == 2 && args[1] == "update" {
    spawn_blocking(move ||
      {update_bin().expect("Error while updating bin")}
    ).await.expect("Error while updating");
  }

  let app_config:AppConfig = AppConfig::new();

  // Init PD
  let pd: PagerDuty = PagerDuty::new(app_config.get_pagerduty_domain(),&app_config.get_pagerduty_api_key()).await;

  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let _ = terminal.draw(|f,|splash_screen(f));

  // create app and run it
  let app = App::new(pd,&app_config).await;
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
